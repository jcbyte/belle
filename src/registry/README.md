All package data is stored under the `$dir_root` path defined in your `belle_config.toml` (/src/config)

```
$dir_root/
├── meta/           <-- Full package details
│   └── {name}/
│       └── {version}.toml
└── manifest/       <-- Subset for resolution
    └── {name}/
        └── {version}.toml
```

- **Metadata:** The "Source of Truth." Contains every detail about the package.
- **Manifests:** Subset of metadata. It contains only the specific fields required for package resolution.
