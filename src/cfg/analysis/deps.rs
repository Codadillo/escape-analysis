use std::collections::{HashMap, HashSet};

use crate::cfg::{Cfg, Statement, Terminator, Value};

use super::Context;

#[derive(Clone, Debug, PartialEq)]
pub struct DepGraph {
    pub nodes: Vec<Node>,
    pub new_lives: HashSet<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub weight: (),
    pub deps: Deps,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Deps {
    All(Vec<usize>),
    Xor(Vec<usize>),
}

impl Node {
    pub fn leaf(weight: ()) -> Self {
        Self {
            weight,
            deps: Deps::All(vec![]),
        }
    }
}

impl Deps {
    pub fn get(&self) -> &Vec<usize> {
        match self {
            Self::All(d) | Self::Xor(d) => d,
        }
    }

    pub fn get_mut(&mut self) -> &mut Vec<usize> {
        match self {
            Self::All(d) | Self::Xor(d) => d,
        }
    }
}

impl DepGraph {
    pub fn from_cfg(ctx: &mut Context, cfg: &Cfg) -> Self {
        let mut this = Self {
            nodes: vec![Node::leaf(()); cfg.place_count],
            new_lives: HashSet::new(),
        };
        this.nodes[0].deps = Deps::Xor(vec![]);

        for stmnt in cfg.statements() {
            match stmnt {
                Statement::Assign(a) => match &a.value {
                    Value::Place(src) => {
                        this.nodes[a.place].deps = Deps::Xor(vec![*src]);
                    }
                    Value::Call { func, args } => match func.0.as_str() {
                        "tuple" => {
                            this.nodes[a.place].deps = Deps::All(args.clone());
                        }
                        "invent" => {}
                        _ => {
                            let fdeps = ctx.compute_depgraph(func).unwrap();
                            this.merge_in(a.place, args, ctx.get_cfg(func).unwrap(), fdeps);
                        }
                    },
                },
                Statement::Nop => {}
            }
        }

        for bb in &cfg.basic_blocks {
            if let Some(Terminator::Return(place)) = bb.terminator {
                let Deps::Xor(deps) = &mut this.nodes[0].deps else {
                    unreachable!()
                };
                deps.push(place);
            }

            for phi in &bb.phi {
                this.nodes[phi.place].deps = Deps::Xor(phi.opts.values().copied().collect());
            }
        }

        for (i, node) in this.nodes.iter().enumerate() {
            if !(1..=cfg.arg_count).contains(&i) && !matches!(node.deps, Deps::Xor(_)) {
                this.new_lives.insert(i);
            }
        }

        this
    }

    pub fn merge_in(
        &mut self,
        parent: usize,
        passed_args: &[usize],
        child_cfg: &Cfg,
        child_graph: DepGraph,
    ) {
        assert_eq!(child_cfg.arg_count, passed_args.len());

        let child_args = 1..=child_cfg.arg_count;
        let mut remap: HashMap<_, _> = child_args
            .clone()
            .zip(passed_args.iter().copied())
            .chain([(0, parent)])
            .collect();

        for (place, mut node) in child_graph.nodes.into_iter().enumerate() {
            let remap_place = self.remap_place(place, &mut remap);

            for dep in node.deps.get_mut() {
                *dep = self.remap_place(*dep, &mut remap);
            }

            if !child_args.contains(&place) {
                self.nodes[remap_place] = node;
            }
        }

        self.new_lives.extend(
            child_graph
                .new_lives
                .into_iter()
                .map(|l| remap.get(&l).unwrap()),
        );
    }

    fn remap_place(&mut self, place: usize, remap: &mut HashMap<usize, usize>) -> usize {
        *remap.entry(place).or_insert_with(|| {
            self.nodes.push(Node::leaf(()));
            self.nodes.len() - 1
        })
    }

    pub fn preorder(&self) -> Vec<usize> {
        let mut ordering = vec![0];
        let mut cursor = 0;

        while cursor != ordering.len() {
            let node = &self.nodes[ordering[cursor]];
            cursor += 1;

            for &dep in node.deps.get() {
                if !ordering.contains(&dep) {
                    ordering.push(dep)
                }
            }
        }

        ordering
    }

    pub fn predecessors(&self) -> HashMap<usize, Vec<usize>> {
        let mut preds: HashMap<usize, Vec<usize>> = HashMap::new();

        for (i, node) in self.nodes.iter().enumerate() {
            for &dep in node.deps.get() {
                preds.entry(dep).or_default().push(i);
            }
        }

        preds
    }

    pub fn simplify(&mut self, args: &[usize]) {
        for i in self.preorder().into_iter().rev() {
            // replace child with grandchild if single-dep xor
            for (j, child) in self.nodes[i].deps.get().clone().into_iter().enumerate() {
                let grandchild = match &self.nodes[child].deps {
                    Deps::Xor(gc) if gc.len() == 1 => gc[0],
                    _ => continue,
                };

                self.nodes[i].deps.get_mut()[j] = grandchild;
            }

            // now do simplifcations that only work on xor
            let Deps::Xor(deps) = &mut self.nodes[i].deps else {
                continue;
            };

            let mut tmp_deps = std::mem::take(deps);

            // flatten nested xors
            let mut dead_deps = vec![];

            for j in 0..tmp_deps.len() {
                let Deps::Xor(depdeps) = &self.nodes[tmp_deps[j]].deps else {
                    continue;
                };

                dead_deps.push(j);
                tmp_deps.extend(depdeps);
            }

            for dead in dead_deps.into_iter().rev() {
                tmp_deps.remove(dead);
            }

            // remove redundant children
            *self.nodes[i].deps.get_mut() = tmp_deps
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
        }

        // have root become child if single-dep xor
        match &self.nodes[0].deps {
            Deps::Xor(deps) if deps.len() == 1 && !args.contains(&deps[0]) => {
                self.nodes[0].deps = self.nodes[deps[0]].deps.clone();

                if let Deps::All(_) = self.nodes[0].deps {
                    self.new_lives.insert(0);
                }
            }
            _ => {}
        }

        // delete hanging xor nodes
        let reachable: HashSet<_> = self.preorder().into_iter().collect();
        let mut remap: Vec<_> = (0..self.nodes.len()).collect();

        for i in (1..self.nodes.len()).rev() {
            if !reachable.contains(&i) {
                self.nodes.remove(i);
                self.new_lives.remove(&i);

                for j in i..remap.len() {
                    remap[j] -= 1;
                }
            }
        }

        for dep in self.nodes.iter_mut().flat_map(|n| n.deps.get_mut()) {
            *dep = remap[*dep];
        }

        self.new_lives = self.new_lives.iter().map(|&l| remap[l]).collect();
    }
}

type Nd = usize;
type Ed = (usize, usize);
impl<'a> dot::Labeller<'a, Nd, Ed> for DepGraph {
    fn graph_id(&self) -> dot::Id<'_> {
        dot::Id::new("DependencyGraph").unwrap()
    }

    fn node_id(&self, n: &Nd) -> dot::Id<'_> {
        dot::Id::new(format!("N{}", n)).unwrap()
    }

    fn node_label<'b>(&self, n: &Nd) -> dot::LabelText<'_> {
        let label = match self.nodes[*n].deps {
            Deps::All(_) => format!("_{n}"),
            Deps::Xor(_) => format!("Xor(_{n})"),
        };

        dot::LabelText::html::<String>(label.into())
    }

    fn node_color(&self, node: &Nd) -> Option<dot::LabelText<'_>> {
        if let Deps::Xor(_) = self.nodes[*node].deps {
            return Some(dot::LabelText::LabelStr("grey".into()));
        }

        Some(match self.new_lives.contains(node) {
            true => dot::LabelText::LabelStr("orange".into()),
            false => dot::LabelText::LabelStr("green".into()),
        })
    }

    fn edge_label<'b>(&self, _: &Ed) -> dot::LabelText<'_> {
        dot::LabelText::LabelStr("".into())
    }

    fn edge_style(&self, (a, _): &Ed) -> dot::Style {
        match self.nodes[*a].deps {
            Deps::Xor(_) => dot::Style::Dashed,
            Deps::All(_) => dot::Style::None,
        }
    }

    fn edge_color(&self, (a, _): &Ed) -> Option<dot::LabelText<'_>> {
        Some(match self.nodes[*a].deps {
            Deps::Xor(_) => dot::LabelText::LabelStr("grey".into()),
            Deps::All(_) => dot::LabelText::LabelStr("black".into()),
        })
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for DepGraph {
    fn nodes(&self) -> dot::Nodes<'a, Nd> {
        (0..self.nodes.len()).collect()
    }

    fn edges(&self) -> dot::Edges<'_, Ed> {
        self.nodes
            .iter()
            .enumerate()
            .flat_map(|(i, n)| n.deps.get().iter().map(move |&d| (i, d)))
            .collect()
    }

    fn source(&self, e: &Ed) -> Nd {
        e.0
    }

    fn target(&self, e: &Ed) -> Nd {
        e.1
    }
}
