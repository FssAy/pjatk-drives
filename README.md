# Pjatk Drives
Pjatk Drives is an SFTP web client for the "Polish-Japanese Academy of Information Technology".

Log-in with your academic credentials to browse the "Public" and "Zet" catalogs.

Entire project is at the moment a "proof of concept" and is not maintained because of my lack of experience in the frontend. If you are interested in bringing this 
project back to life, contact me, create an issue, or whatever.

There is a lot of work to do on the code, but I don't think this project has a future. That's why I am not keen on fixing everything right now.

[click](https://www.youtube.com/watch?v=pnYNM4pzFJQ) to see presentation of the Pjatk Drives.

# Config 
The configuration file is in JSON format and will be created automatically on the server startup.

- `bind [string]` - An address on which the server will be listening for the requests.
- `host [string]` - The base HTTP address.
- `api_version [u8]` - The default api version to use.
- `ftp_host [string]` - Address of the SFTP server to connect to.
- `ftp_domain [string]` - Not used.

# API
the api format is: <br>
`<HOST>/api/<API_VERSION>/`

If any error occurs the respons data is an JSON object which structure can be found in the "crate::handler::responses::ErrorMessage".

Any request related to the SFTP protocol is fully dependent on the external system and the response can take up to 20 seconds to receive, but that has no impact onto other clients.

### /login
**_Methods_:** POST <br>
**_Description_:** Creates the SFTP client for a specific user and returns it's id as the body and in the set-cookie header. <br>
**_Requirements_:** JSON object in the body with the credentials (`"user"`, `"password"`) <br>
**_Notes_:** Any SFTP client will be logged-off after 5 minutes of inactivity. 

### /ftp<FTP_PATH>
**_Methods_:** GET, POST, PATCH <br>
**_Description_:** Handles any ftp file related operations. <br>
**_Requirements_:** SFTP client id in the "ftp" header or in the cookie. Valid <FTP_PATH>. <br>
**_Optional_:** "is-dir" header (true, false). "as-html" header (true, false). <br>
**_Notes_:** Only GET method is fully implemented and allows to list the ftp directory and download a file.

In the place of `<FTP_PATH>` in the uri it's required to put a valid path to the file or diretory.
For example `/ftp/` will list out everything in the root directory.
`/ftp/public` will list out everything in the public directory.
And the `/ftp/zet/my file.txt` will download the "my file.txt" file.

In case if a directory has a `.` or a file doesn't include any extension it is good to use "is-dir" header which will help in this situation. <br>
Here is a table presenting what the server will do based on value of the "is-dir" header and if name of the entity contains an extension:

| contains extension | is-dir    | **prediction** |
|--------------------|-----------|---------------:|
| true               | undefined |         _file_ |
| true               | false     |         _file_ |
| false              | false     |         _file_ |
| true               | true      |          _dir_ |
| false              | undefined |          _dir_ |
| false              | true      |          _dir_ |

"as-html" header is used to return the listing response in the html format expected by the web server.

Downloading file is divided into multiple stages. The file transfer will be closed after 30 seconds of inactivity.

**_Responses_:** 
- Listing directory - If "as-html" is not present the response will be in the JSON format. Structure can be found at "crate::handler::endpoints::ftp::listing::Listing".
- Downloading file - The response is in the JSON format. Structure can be found at "crate::cache::ftp::transfer::FileContentPack".

# HTML
Entire frontend is embedded into the binary in compile-time. 

Some html files require editing in the runtime. These files contain variables in the format of `{{var}}` which will be replaced 
with proper value by the server.

