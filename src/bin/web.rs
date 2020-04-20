use stdweb::js;
use stdweb::web::*;

fn main() {
    let msg = "Hello world!";
    js! {
        console.log(@{msg})
    }
}
