use op_engine::*;

fn main() {
    let mut s = Session::new();

    s.record_start(0);
    s.record_append(&[1.0, 2.0, 3.0, 4.0]);
    s.record_end();

    s.record_start(2);
    s.record_append(&[5.0, 6.0, 7.0, 8.0]);
    s.record_end();

    s.record_start(16);
    s.record_append(&[1.0, 2.0, 3.0, 4.0]);
    s.record_end();

    println!("{:?}", s.render_all());
}
