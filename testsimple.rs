fn identity(a) {
    a
}

fn my_function(a, b, c) {
    let input = identity(a);

    let hello = if input {           
        identity(c)              
    } else {
        identity(c)  
    };

    tuple(input, input, hello)   
}
