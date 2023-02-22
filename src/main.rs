mod vm;

use vm::*;

fn main() {
    let mut test_chunk = Chunk::new();
    test_chunk.push(OpCode::RETURN, 1);

    for i in 0..260 {
        test_chunk.push_constant(i, 2);
    }

    println!("{:?}", test_chunk);
}
