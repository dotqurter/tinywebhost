# tinywebhost
This is my implementation of the Rust tutorial webserver, intended to host my site.

## Usage
Compile with `cargo build`.
To set the IP address and port, run the program with the `IP_ADDR` environment variable set in the style of `0.0.0.0:80`.

To set the root folder, run the program with the `ROOT_FOLDER` environment variable set in the style of `/path/to/root` (unix derivatives) or `C:\\Path\\to\\root` (windows).

### Example
Powershell: `$Env:ROOT_FOLDER="C:\website\"; $Env:IP_ADDR="127.0.0.1:80"; cargo run --release`

Bash: `$ROOT_FOLDER="/var/web/" $IP_ADDR="127.0.0.1:80" cargo run --release`
