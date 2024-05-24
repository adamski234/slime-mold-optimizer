fn main() {
    let mut a = vec![5.0, 2.0, 1.0, 4.0, 3.0];
    a.sort_by(|l, r| l.partial_cmp(r).unwrap());
    println!("{:#?}",  0.0 == -0.0f32);
}
