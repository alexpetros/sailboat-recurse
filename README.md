# Project Sailboat

Run your ActivityPub presence from your own website.

## Project Goals

### Main Ideas
* Create a web server with both a authenticated "owner" view and a public view
* The owner can see the feeds they're subscribed to and post to their own feeds
* Creating new feeds should be extremely simple
* Unauthenticated viewers of the website are presented with a nice feed view, and a copyable link to subscribe
* All data is stored in a SQLite DB

### Stretch Concepts
* Client-server implementation would allow people to login to their websites via Mastodon clients
* Shouldn't be too hard to just add regular web server stuff, and serve all this off a subdomain or a slash

