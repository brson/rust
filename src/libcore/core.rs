export repeat;

/*
Function: repeat

Execute a function for a set number of times
*/
fn repeat(times: uint, f: block()) {
    let i = 0u;
    while i < times {
        f();
        i += 1u;
    }
}