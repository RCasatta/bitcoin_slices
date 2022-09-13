
# Bitcoin slices

ZERO allocations parse library for Bitcoin data structures.

## Tradeoffs

Check the CONS before using this library, use rust-bitcoin if they are too restrictive for your case.

### Pros

* Deserialization is amazingly fast, since no allocation is made during parsing.
* Serialization is incredibly fast, since a slice of the serialized data is kept in the structure.
* Just one, optionally removable, dependencies for calculating Block and Transaction hash
* No standard
* hashing a little faster because slice are ready without re-serializing 

### Cons

* Full data must be in memory, there is no streaming (Read/Write) API
* Data structure are read-only, cannot be modified

## Previous work and credits

* bitiodine parsing/ credit to the guy
* Witness PR
* corallo comment

## TODO

- [x] Transaction iteration in Block, 
- [x] inputs and outputs iteration in Transaction, 
- [x] methods on tx that return slices (or tuples of slice) of the preimage of txid()
- [x] txid() with optional feature bitcoin_hashes
- [x] no std
- [ ] write Readme
- [ ] write documentation