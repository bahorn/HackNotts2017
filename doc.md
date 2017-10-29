# Author
B. Horn <b <AT> horn <DOT> uk>

# DomainFrontingThing
* main backend is written in Rust,with Rocket+Diesel+Postgres
* Some python to upload to s3.
* And shell scripts to do the actual magic.
* (And docker for running a postgres instance)

* it is purely CLI right now. :)







# Why?
Useful in for bypassing censorship in some countries.
Turns out blocking major CDNs cause a lot of issues.

I wrote an example that uses it to distribute an API key,
and another to give out a Tor Bridge.

