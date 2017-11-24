extern crate radix_trie;

use self::radix_trie::Trie;

pub struct Ruler<'a>{
    domain_trie: Trie<&'a str, &'a str>,
}

impl <'a> Ruler<'a>{
    pub fn new()-> Ruler<'a> {
        let mut domain_trie= Trie::new();
        domain_trie.insert("a", "v");
        let ruler = Ruler {
            domain_trie: domain_trie,
        };

        ruler
    }

    pub fn rule_domain(&self, domain: &str) -> Option<&&str> {
        self.domain_trie.get_ancestor_value(domain)
    }
}

