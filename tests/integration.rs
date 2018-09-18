extern crate wikipedia_externallinks_fast_extraction;

use wikipedia_externallinks_fast_extraction::iter_string_urls;

#[test]
fn frwiki_two_urls() {
    let dump: &[u8] = br#"
CREATE TABLE `externallinks` (
  `el_id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `el_from` int(8) unsigned NOT NULL DEFAULT '0',
  `el_to` blob NOT NULL,
  `el_index` blob NOT NULL,
  `el_index_60` varbinary(60) NOT NULL,
  PRIMARY KEY (`el_id`),
  KEY `el_from` (`el_from`,`el_to`(40)),
  KEY `el_to` (`el_to`(60),`el_from`),
  KEY `el_index` (`el_index`(60)),
  KEY `el_index_60` (`el_index_60`,`el_id`),
  KEY `el_from_index_60` (`el_from`,`el_index_60`,`el_id`)
) ENGINE=InnoDB AUTO_INCREMENT=44782967 DEFAULT CHARSET=binary ROW_FORMAT=COMPRESSED KEY_BLOCK_SIZE=8;

INSERT INTO `externallinks` VALUES
    (1,599334,'http://example.com/','',''),
    (2,100161,'http://example2.com/','','')
;
    "#;
    let urls: Vec<_> = iter_string_urls(dump).flatten().collect();
    let expected_urls = vec![
        "http://example.com/".to_string(),
        "http://example2.com/".to_string()
    ];
    assert_eq!(expected_urls, urls);
}