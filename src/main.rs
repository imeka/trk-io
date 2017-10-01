
mod trk;
mod streamlines;

fn main() {
    let path = "C:\\Users\\Nil\\Desktop\\DCC10__whole_brain_wm_tracts.trk";
    // let path = "D:\\Data\\MI-Brain\\rat\\trk_normal.trk";
    let streamlines = trk::read_streamlines(path);
    /*for streamline in &streamlines {
        for point in streamline {
            println!("{} {} {}", point.x, point.y, point.z);
        }
        println!("")
    }*/

    trk::write_streamlines(&streamlines, "C:\\Imeka\\rust_trk\\trk.trk");
}
