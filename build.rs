// build.rs roda ANTES da compilação do crate.
// napi_build::setup() injeta as flags de linkagem necessárias para que o
// binário gerado seja carregável como um addon nativo do Node (N-API).
extern crate napi_build;

fn main() {
    napi_build::setup();
}
