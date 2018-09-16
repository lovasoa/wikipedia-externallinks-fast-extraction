# wikipedia-externallinks-fast-extraction

Fast extraction of all external links from wikipedia.

This small program takes a wikipedia SQL dump on the input, and 
outputs an exhaustive list of all outlinks from wikipedia to other sites.

## How to use

### On linux

#### Download
```sh
wget 'https://github.com/lovasoa/wikipedia-externallinks-fast-extraction/releases/download/0.1.2/wikipedia-externallinks-fast-extraction'
chmod +x wikipedia-externallinks-fast-extraction
```

#### Extract links

```sh
LANG=en
curl 'https://dumps.wikimedia.org/'$LANG'wiki/latest/'$LANG'wiki-latest-externallinks.sql.gz' |
	gunzip |
	./wikipedia-externallinks-fast-extraction > urls.txt
```

The urls will be streamed into urls.txt as they are downloaded.
You will see warnings on the standard error.
Most of them should be safe to ignore.
