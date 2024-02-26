use std::{
    fs::{self, File},
    io::{self, Read, Write},
};
mod core;

fn main() -> io::Result<()> {
    let mut app: core::System = core::System::new()?;
    app.login()?;
    app.app();
    app.curr_user_exercises();

    // let mut buffer = File::create("foo.txt")?;
    // buffer.write_all(b"lol")?;
    // let mut strbuf = String::new();
    // let mut profiles_handle = match File::open("foo.txt") {
    //     Ok(file) => file,
    //     Err(e) => return Err(e),
    // };

    Ok(())
}
