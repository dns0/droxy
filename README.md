fast rule-based dns proxy


it inspects the dns request and depending on what domain name is being queried, forwards the request to an upstream server of your choice


with it, you can:

send queries of local domains, such as **.lan, **.local to your router as home

query some domains that you don't want others to know using only safe upstream servers

take advantage of opennic to resolve alternative root systems, such as .bit

use the dns of your isp for other domains for speed


Configuration is simple and examples are in config:

basically, resolve.config is in Toml and defines what upstream dns servers to use for each region.

regions are defined in folders called region.name, the part after "region." is the name of the region
all files in the folders contain domains written in reverse notation, for example, com.example.www

domains are matched to regions in a longest prefix way, for example you can configure com.example to use one server, and if you configure com.example.static to use a different server, its sub names also use that different server

