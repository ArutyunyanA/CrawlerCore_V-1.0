# Asyncronic crawler for pentest and parsing domains

## Usage

```bash


cargo run crawler -h

███████╗██╗  ██╗██╗███╗   ██╗ ██████╗ ██████╗ ██╗
██╔════╝██║  ██║██║████╗  ██║██╔═══██╗██╔══██╗██║
███████╗███████║██║██╔██╗ ██║██║   ██║██████╔╝██║
╚════██║██╔══██║██║██║╚██╗██║██║   ██║██╔══██╗██║
███████║██║  ██║██║██║ ╚████║╚██████╔╝██████╔╝██║
╚══════╝╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═════╝ ╚═╝
                                                 


Usage:
    crawler [OPTIONS] <url> [url...]

Options:
    --crawling <N>      Crawling concurrency (default: 4)
    --process <N>    Processing concurrency (default: 2)
    --delay <ms>     Delay Between requests (default: 100)
    --robots, -r     Apply robots.txt rules for scheduling
    --help           Show this help
```

## Crawling

```bash
cargo run http://127.0.0.1:3000
warning: `crawler` (bin "crawler") generated 5 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.12s
     Running `target/debug/crawler 'http://127.0.0.1:3000'`
███████╗██╗  ██╗██╗███╗   ██╗ ██████╗ ██████╗ ██╗
██╔════╝██║  ██║██║████╗  ██║██╔═══██╗██╔══██╗██║
███████╗███████║██║██╔██╗ ██║██║   ██║██████╔╝██║
╚════██║██╔══██║██║██║╚██╗██║██║   ██║██╔══██╗██║
███████║██║  ██║██║██║ ╚████║╚██████╔╝██████╔╝██║
╚══════╝╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═════╝ ╚═╝
                                                 

[+] Target domains loaded: 1
[+] Seed URLs queued: 1
[javascript] - http://127.0.0.1:3000/polyfills.js
[javascript] - http://127.0.0.1:3000/scripts.js
[javascript] - http://127.0.0.1:3000/main.js
[url] - [code-200] - http://127.0.0.1:3000/ftp
[href] - http://127.0.0.1:3000/
[href] - http://127.0.0.1:3000/ftp
[href] - http://127.0.0.1:3000/ftp/quarantine
[href] - http://127.0.0.1:3000/ftp/acquisitions.md
[href] - http://127.0.0.1:3000/ftp/announcement_encrypted.md
[href] - http://127.0.0.1:3000/ftp/coupons_2013.md.bak
[href] - http://127.0.0.1:3000/ftp/eastere.gg
[href] - http://127.0.0.1:3000/ftp/encrypt.pyc
[href] - http://127.0.0.1:3000/ftp/incident-support.kdbx
[href] - http://127.0.0.1:3000/ftp/legal.md
[href] - http://127.0.0.1:3000/ftp/package-lock.json.bak
[href] - http://127.0.0.1:3000/ftp/package.json.bak
[href] - http://127.0.0.1:3000/ftp/suspicious_errors.yml
[url] - [code-200] - http://127.0.0.1:3000/ftp/acquisitions.md
[url] - [code-200] - http://127.0.0.1:3000/scripts.js
[url] - [code-200] - http://127.0.0.1:3000/
[javascript] - http://127.0.0.1:3000/polyfills.js
[javascript] - http://127.0.0.1:3000/scripts.js
[javascript] - http://127.0.0.1:3000/main.js
[url] - [code-200] - http://127.0.0.1:3000/polyfills.js
[url] - [code-200] - http://127.0.0.1:3000/scripts.js
[linkfinder] - http://127.0.0.1:3000/rest/admin/application-configuration
[linkfinder] - http://127.0.0.1:3000/chunk-LHKS7QUN.js
[linkfinder] - http://127.0.0.1:3000/chunk-T3PSKZ45.js
[linkfinder] - http://127.0.0.1:3000/chunk-TWZW5B45.js
[href] - http://127.0.0.1:3000/tokens
[href] - http://127.0.0.1:3000/token/0xdac17f958d2ee523a2206206994597c13d831ec7#balances#flow
[href] - http://127.0.0.1:3000/nft-top-contracts
[href] - http://127.0.0.1:3000/nft-top-mints
[href] - http://127.0.0.1:3000/nft-trades
[href] - http://127.0.0.1:3000/nft-transfers
[href] - http://127.0.0.1:3000/nft-latest-mints
```
