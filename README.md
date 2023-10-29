# IliasUploaderUtility
*Because Ilias is a UX Nightmare*

---
A tool to upload files to (KIT-)Ilias

## Installation
Either
- download the binary from the releases page
- Clone the repo and run `cargo install` (obviously requires [cargo](https://github.com/rust-lang/cargo))
If you are on windows you can also add a context menu entry `upload to ilias` to the explorer by executing [`windows_contextmenu.reg`](./windows_contextmenu.reg)
- Use nix flakes: `nix run github:Mr-Pine/IliasUploaderUtility`

## Usage
#### CLI
To upload a file to ilias run `ilias_uploader_utility --ilias-id [The ref_id of the ilias Upload page (see below)] --username [your u-Name] --password [mySuperS3curePassw0rd1337]`. If you don't provide a password it will prompt you and save it.

#### Config files
You can also provide config files by putting a file named `.ilias_upload` ([See example](./.ilias_upload)) in the same, or a parent-, directory as where you execute the command. By default search depth is `3`, meaning it will search `.`. `..`, `../..`. You can change the depth by providing `-d [your favorite depth]`

#### `ilias_id`
![ilias_id example](./Media/ilias_id.png)
An example to where to find your ilias_id
