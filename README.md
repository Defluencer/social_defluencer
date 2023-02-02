# Social Defluencer Website

### How To Build Web App
- Download latest [Bulma](https://bulma.io/) version.
- Download latest [FontAwesome](https://fontawesome.com/download) version. 
- Rename folders to ```bulma``` & ```fontawesome```.
- Copy both to root folder.
- Install [Trunk](https://trunkrs.dev/)
- Run ```trunk build --release```
- ```ipfs add --cid-version=1 --chunker=size-1048576 -r www```

### How To Build Tauri App
- Download latest [Bulma](https://bulma.io/) version.
- Download latest [FontAwesome](https://fontawesome.com/download) version. 
- Rename folders to ```bulma``` & ```fontawesome```.
- Copy both to root folder.
- Install [Tauri](https://tauri.app/v1/guides/getting-started/prerequisites#setting-up-linux)
- Install [Trunk](https://trunkrs.dev/)
- Run ```cargo tauri build ```