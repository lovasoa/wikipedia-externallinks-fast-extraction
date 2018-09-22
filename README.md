# wikipedia-externallinks-fast-extraction

Fast extraction of all external links from wikipedia.

Wikipedia provides various dumps, including a dump of all the links
from it to other sites. Unfortunately, the only format in which this data
can be doawnloaded is SQL, which is not very practical. Typically, you would
have to set up a MySQL database, and then import all the data from the dump in it
before being able to work with the data.

This project provides a simple binary that parses the SQL file
in a streaming fashion, and outputs the links as plain text, one link per line.
It requires no setup, and is very fast (you will be limited by the speed at which
you can download the dumps, not by this program).
It may be useful to create a seedlist for a crawler.

This small program takes a wikipedia SQL dump on the input, and 
outputs an exhaustive list of all outlinks from wikipedia to other sites.

## How to use

### On linux

#### Download
```sh
wget 'https://github.com/lovasoa/wikipedia-externallinks-fast-extraction/releases/download/0.1.5/wikipedia-externallinks-fast-extraction'
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
