# v2fly-domains
This project aims to convert v2fly geosite format into sing-box format and create Clash ruleset for the use in Shadowrocket.

Based on [v2fly/domain-list-community](https://github.com/v2fly/domain-list-community) and [Loyalsoldier/surge-rules](https://github.com/Loyalsoldier/surge-rules)

# Usage
 * Xray/V2ray geosite: https://github.com/Raudeck/v2fly-domains/releases/latest/download/geosite.dat
 * Sing-box:
    * Example (binary):
        ```
        "ruleset": [
            {
                "tag": "cn",
                "type": "remote",
                "format": "binary",
                "url": "https://raw.githubusercontent.com/Raudeck/v2fly-domains/refs/heads/singbox/cn.srs",
            }
        ]
        ```
* Clash:
    * Example (domain):
        ```
        rule-providers:
            cn:
                type: http
                behavior: domain
                url: "https://raw.githubusercontent.com/Raudeck/v2fly-domains/refs/heads/domain/cn.list"
                path: ./ruleset/cn.list
                interval: 86400
        ```
    * Example (classical):
        ```
        rule-providers:
            cn:
                type: http
                behavior: classical
                url: "https://raw.githubusercontent.com/Raudeck/v2fly-domains/refs/heads/classical/cn.list"
                path: ./ruleset/cn.list
                interval: 86400
        ```