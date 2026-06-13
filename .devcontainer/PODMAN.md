# Podman-specific settings

## Using with Podman Compose

1. Install podman-compose:
   ```bash
   pip install podman-compose
   ```

2. Or use Podman directly with Docker-compatible commands:
   ```bash
   # Enable Docker-compatible socket
   systemctl --user enable podman.socket
   ```

## Podman Compose Commands

```bash
# Start containers
podman-compose -f .devcontainer/docker-compose.yml up -d --build

# Stop containers
podman-compose -f .devcontainer/docker-compose.yml down

# View logs
podman-compose -f .devcontainer/docker-compose.yml logs -f

# Build the dev image
podman-compose -f .devcontainer/docker-compose.yml build
```

## Podman-specific Notes

- Podman Compose may require `version: "3"` format instead of `"3.8"`
- The `init` property in docker-compose.yml may not be supported in podman-compose - remove if needed
- Volume mounts use slightly different syntax in some cases
- For rootless Podman, run: `systemctl --user start podman.socket`

### Troubleshooting

If you encounter issues with the `init` property, edit `docker-compose.yml` to remove:
```yaml
init: true
```

This is optional and can be safely removed for Podman compatibility.