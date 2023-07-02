# Status

Deployed at https://status.sachiniyer.com.

Just used to check the status of my currently deployed applications. Consumed by https://sachiniyer.com 

Was originally built to avoid cors errors on my main website, but also is just a nice utility to check what is up.

## Features

1. Pulls from my [nginx conf](https://github.com/sachiniyer/cheap_portable_k3s/blob/main/nginx.conf) to figure out what applications are currently exposed. Then does GET requests to make sure that they are active. 
2. Exposes a websocket endpoint at https://status.sachiniyer.com/ws - this is best, because you will get information as soon as a response comes in
3. Exposes a regular HTTP endpoint at https://status.sachiniyer.com - this is a bit slower
4. Packaged as a docker container at [sachiyer/status](https://hub.docker.com/repository/docker/sachiyer/status/general). I use github actions to automate that deployment.

## How it's build

Everything is written in async with tokio, axum, and reqwest. 

I use async and multi-threading to send out all the requests with reqwest. Then for the websocket version, I send back the results as soon as the threads complete. For the web version, I collect results from all the threads and then send it out as an HTTP response. The websocket version is what is used on my main website. 

