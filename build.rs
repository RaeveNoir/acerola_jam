extern crate embed_resource;

fn main() {
    embed_resource::compile("assets/manifest.rc", embed_resource::NONE);
}
