# fast rule-based dns proxy


it inspects the dns request and depending on what domain name is being queried, forwards the request to an upstream server of your choice


# with it, you can:

- send queries of local domains, such as **.lan, **.local to your router as home

- query some domains that you don't want others to know using only safe upstream servers

- take advantage of opennic to resolve alternative root systems, such as .bit

- use the dns of your isp for other domains for speed


# Configuration

it is really simple and examples are in the folder `config`

basically, `resolve.config` is in Toml and defines what upstream dns servers to use for each region.

regions are defined in folders called `region.`_name_, for example, you can create a region for your home in a folder `region.home`.
Files in the folders contain domains written in reverse notation, for example, com.example.www, uk.co.google.www

domains are matched to regions in a longest prefix way, for example com.example matches com.example.www, com.example.support. But if you want com.example.static to use a different server, just configure it, and because it's longer, it overrides the configuration for com.example.

# Commandline arguments

there is one, `-c` or `--config` for setting the folder container configuration files. By default it's `config` in the current folder.
