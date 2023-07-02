# Status

Deployed at https://status.sachiniyer.com.

Just used to check the status of my currently deployed applications. Consumed by https://sachiniyer.com 

Was originally built to avoid cors errors on my main website, but also is just a nice utility.

# Features

Pulls from my [nginx conf](https://github.com/sachiniyer/cheap_portable_k3s/blob/main/nginx.conf) to figure out what applications are currently exposed. Then does GET requests to make sure that they are active. 

Exposes a websocket endpoint at https://status.sachiniyer.com/ws and a regular endpoint at https://status.sachiniyer.com . The websocket endpoint is really what is best, because you will get information as soon as the get request comes back. The regular HTTP endpoint is much slower. Everything is written in async with tokio, axum, and reqwest. 

