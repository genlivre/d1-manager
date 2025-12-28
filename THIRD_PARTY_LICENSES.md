# Third Party Licenses

D1 Manager uses open source software. Below is a list of the main dependencies and their licenses.

## License Summary

| License | Count |
|---------|-------|
| MIT | 121 |
| Apache-2.0 OR MIT | 309 |
| Apache-2.0 | 15 |
| BSD-2-Clause | 1 |
| BSD-3-Clause | 3 |
| BSL-1.0 | 2 |
| ISC | 3 |
| MPL-2.0 | 1 |
| Zlib | 3 |
| Unicode-3.0 | 18 |
| OFL-1.1 (fonts) | 1 |

## Main Dependencies

### GUI Framework

- **egui** (MIT OR Apache-2.0) - https://github.com/emilk/egui
  - An easy-to-use immediate mode GUI in pure Rust
- **eframe** (MIT OR Apache-2.0) - https://github.com/emilk/egui
  - The egui framework for native and web apps
- **egui_extras** (MIT OR Apache-2.0) - https://github.com/emilk/egui
  - Extra widgets for egui
- **winit** (Apache-2.0) - https://github.com/rust-windowing/winit
  - Cross-platform window creation and management

### Networking

- **reqwest** (MIT OR Apache-2.0) - https://github.com/seanmonstar/reqwest
  - An HTTP Client for Rust
- **tokio** (MIT) - https://github.com/tokio-rs/tokio
  - An asynchronous runtime for Rust
- **rustls** (Apache-2.0 OR ISC OR MIT) - https://github.com/rustls/rustls
  - A modern TLS library in Rust

### Serialization

- **serde** (MIT OR Apache-2.0) - https://github.com/serde-rs/serde
  - A generic serialization/deserialization framework
- **serde_json** (MIT OR Apache-2.0) - https://github.com/serde-rs/json
  - JSON support for Serde

### Database

- **rusqlite** (MIT) - https://github.com/rusqlite/rusqlite
  - SQLite bindings for Rust
- **libsqlite3-sys** (MIT) - https://github.com/rusqlite/rusqlite
  - Native bindings to the SQLite library

### Security

- **keyring** (MIT OR Apache-2.0) - https://github.com/hwchen/keyring-rs
  - Cross-platform library for managing passwords/secrets
- **zeroize** (MIT OR Apache-2.0) - https://github.com/RustCrypto/utils
  - Securely zero memory while avoiding compiler optimizations

### Utilities

- **dirs** (MIT OR Apache-2.0) - https://github.com/soc/dirs-rs
  - Low-level library for standard directories
- **rfd** (MIT) - https://github.com/PolyMeilex/rfd
  - Rusty File Dialogs
- **open** (MIT) - https://github.com/Byron/open-rs
  - Open a path or URL using the system-configured program
- **image** (MIT OR Apache-2.0) - https://github.com/image-rs/image
  - Image processing library

### Text Processing

- **regex-lite** (MIT OR Apache-2.0) - https://github.com/rust-lang/regex
  - A lightweight regex engine
- **sys-locale** (MIT OR Apache-2.0) - https://github.com/1Password/sys-locale
  - Retrieve the active locale on the system

---

## Full License Texts

### MIT License

```
MIT License

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### Apache License 2.0

```
                              Apache License
                        Version 2.0, January 2004
                     http://www.apache.org/licenses/

TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

1. Definitions.

"License" shall mean the terms and conditions for use, reproduction,
and distribution as defined by Sections 1 through 9 of this document.

"Licensor" shall mean the copyright owner or entity authorized by
the copyright owner that is granting the License.

"Legal Entity" shall mean the union of the acting entity and all
other entities that control, are controlled by, or are under common
control with that entity.

"You" (or "Your") shall mean an individual or Legal Entity
exercising permissions granted by this License.

"Source" form shall mean the preferred form for making modifications,
including but not limited to software source code, documentation
source, and configuration files.

"Object" form shall mean any form resulting from mechanical
transformation or translation of a Source form, including but
not limited to compiled object code, generated documentation,
and conversions to other media types.

"Work" shall mean the work of authorship, whether in Source or
Object form, made available under the License.

"Derivative Works" shall mean any work, whether in Source or Object
form, that is based on (or derived from) the Work.

"Contribution" shall mean any work of authorship submitted to the
Licensor for inclusion in the Work.

"Contributor" shall mean Licensor and any Legal Entity on behalf
of whom a Contribution has been received by Licensor.

2. Grant of Copyright License. Subject to the terms and conditions of
this License, each Contributor hereby grants to You a perpetual,
worldwide, non-exclusive, no-charge, royalty-free, irrevocable
copyright license to reproduce, prepare Derivative Works of,
publicly display, publicly perform, sublicense, and distribute the
Work and such Derivative Works in Source or Object form.

3. Grant of Patent License. Subject to the terms and conditions of
this License, each Contributor hereby grants to You a perpetual,
worldwide, non-exclusive, no-charge, royalty-free, irrevocable
patent license to make, have made, use, offer to sell, sell, import,
and otherwise transfer the Work.

4. Redistribution. You may reproduce and distribute copies of the
Work or Derivative Works thereof in any medium, with or without
modifications, and in Source or Object form, provided that You
meet the following conditions:

(a) You must give any other recipients of the Work or
    Derivative Works a copy of this License; and

(b) You must cause any modified files to carry prominent notices
    stating that You changed the files; and

(c) You must retain, in the Source form of any Derivative Works
    that You distribute, all copyright, patent, trademark, and
    attribution notices from the Source form of the Work; and

(d) If the Work includes a "NOTICE" text file, You must include
    a readable copy of the attribution notices contained within
    such NOTICE file.

5. Submission of Contributions. Unless You explicitly state otherwise,
any Contribution intentionally submitted for inclusion in the Work
shall be under the terms and conditions of this License.

6. Trademarks. This License does not grant permission to use the trade
names, trademarks, service marks, or product names of the Licensor.

7. Disclaimer of Warranty. Unless required by applicable law or
agreed to in writing, Licensor provides the Work on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND.

8. Limitation of Liability. In no event shall any Contributor be
liable to You for any damages, including any direct, indirect, special,
incidental, or consequential damages arising from this License or
the use or inability to use the Work.

9. Accepting Warranty or Additional Liability. While redistributing
the Work, You may offer and charge a fee for acceptance of support,
warranty, indemnity, or other liability obligations.

END OF TERMS AND CONDITIONS
```

### BSD 2-Clause License

```
Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice,
   this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
POSSIBILITY OF SUCH DAMAGE.
```

### BSD 3-Clause License

```
Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice,
   this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its
   contributors may be used to endorse or promote products derived from
   this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
ARE DISCLAIMED.
```

### Mozilla Public License 2.0 (MPL-2.0)

```
This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.
```

### ISC License

```
Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
```

### Zlib License

```
This software is provided 'as-is', without any express or implied
warranty. In no event will the authors be held liable for any damages
arising from the use of this software.

Permission is granted to anyone to use this software for any purpose,
including commercial applications, and to alter it and redistribute it
freely, subject to the following restrictions:

1. The origin of this software must not be misrepresented; you must not
   claim that you wrote the original software. If you use this software
   in a product, an acknowledgment in the product documentation would be
   appreciated but is not required.

2. Altered source versions must be plainly marked as such, and must not be
   misrepresented as being the original software.

3. This notice may not be removed or altered from any source distribution.
```

### Boost Software License 1.0 (BSL-1.0)

```
Permission is hereby granted, free of charge, to any person or organization
obtaining a copy of the software and accompanying documentation covered by
this license (the "Software") to use, reproduce, display, distribute,
execute, and transmit the Software, and to prepare derivative works of the
Software, and to permit third-parties to whom the Software is furnished to
do so, all subject to the following:

The copyright notices in the Software and this entire statement, including
the above license grant, this restriction and the following disclaimer,
must be included in all copies of the Software, in whole or in part, and
all derivative works of the Software, unless such copies or derivative
works are solely in the form of machine-executable object code generated by
a source language processor.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE, TITLE AND NON-INFRINGEMENT.
```

### SIL Open Font License 1.1 (OFL-1.1)

The default fonts included in egui are licensed under the SIL Open Font License 1.1.
See https://scripts.sil.org/OFL for the full license text.

---

## Generating a Complete List

To generate a complete list of all dependencies and their licenses, run:

```bash
cargo license
```

Or for JSON output:

```bash
cargo license --json
```

---

*This file was last updated: December 2024*
