use ivory::externs::printf;
use ivory::ivory_export;

#[ivory_export]
fn imported_fn() {
    printf("imported");
}
