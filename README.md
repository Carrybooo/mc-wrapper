This repo contains a simple wrapper for a minecraft server.

# Here's how it works : 

- start wrapper process
  - launch the serverstart.sh process
  - capture stdin/out/err
    - redirect stdout and stderr to wrapper process (1 thread for each)
  - setup a handler for SIGINT and SIGTERM signals on the wrapper process
  - When a SIGINT or SIGTERM is sent to wrapper process :
    - send "stop" litteral command to stdin of server process
    - wait for server to print "Restarting automatically" (which means the server has gracefully stopped and is going to restart)
    - send a ctrl-c signal (SIGINT) to avoid the restart
  - once the SIGINT has been sent :
    - wait for the serverstart.sh to finish
  - terminate the wrapper process.

---

# Usage

To make it run, you can edit a little bit the code to make it fit your use (like the name/path of the file, the log line of restart, etc)   

Then you'll have to compile the code with a `cargo build`. (The binary will likely be in `target/debug/mc-wrapper`)   

And finally copy (or make a link to) the binary in your server folder, and run it !   

You can then send a SIGINT or SIGTERM to your wrapper, and it will shutdown the server gracefully before exiting ! That means it can be used in a Docker image for kubernetes, and your server will still shutdown properly even if you kill the pod !
