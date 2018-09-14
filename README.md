# wikipedia-externallinks-fast-extraction

Fast extraction of all external links from wikipedia

## How to use

On linux:

```sh
LANG=en
wget 'https://github.com/lovasoa/wikipedia-externallinks-fast-extraction/releases/download/0.1.1/wikipedia-externallinks-fast-extraction'
curl 'https://dumps.wikimedia.org/'$LANG'wiki/latest/'$LANG'wiki-latest-externallinks.sql.gz' |
	gunzip |
	./wikipedia-externallinks-fast-extraction > urls.txt
```

The urls will be streamed into urls.txt as they are downloaded.
