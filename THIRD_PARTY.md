# License scope and upstream material

The root MIT grant covers original project code and documentation for which Anthony Diaz holds the rights. Dependencies retain their upstream licenses. Preserve copyright and license notices distributed with those dependencies or any copied source.

The repository also holds source snapshots, research documents, and compressed archives. For copied material, consult the identified upstream revision and any notices shipped with the source. Where an archive or copied item has no documented reuse terms, use the upstream source or obtain permission from its rights holder before redistributing it.

- The [source provenance register](consolidated/SOURCE_PROVENANCE.md) and [blob manifest](consolidated/PROVENANCE_BLOBS.tsv) identify copied source from the N32 solver and the independent NS Lean project.
- The primary Lean package pins external dependencies in [lake-manifest.json](lean/lake-manifest.json). Lake fetches those dependencies separately.
- Historical bundles are indexed in the [workspace manifest](WORKSPACE_MANIFEST.md).
