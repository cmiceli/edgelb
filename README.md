# Getting Started

This is a super simple implementation of an edge loadbalancer which uses a config in the src directory to compile
to wasm (which can then be pushed to remote via `wrangler`.

# Features
 - match based on both header existence and path regex
 - proxy to downstream servers or return a redirect

# Config

The sample is available in the src directory, it is generated from the python file in the demo directory.

The best bet is to change the demo python file and run `python cfg.py > ../src/config.json`

# Usage

This will be made available as a library so that all a user will need to do is the include bytes section 
and then call some function which aligns with the cloudflare worker.rs interface.
