# duo-hotp-export

An exporter for Duo Mobile 2FA codes into actually-free apps. Basically, I don't trust Cisco's Duo Mobile app to run on my own phone.

See also: the [accompanying blog post](https://som.codes/blog/2021-06-08/duo-mobile-hotp/).

## Usage

1. Go to the Duo Security portal
2. Try to register a new Android tablet
3. Save the QR code that the portal gives you.
4. Run the program:

```shell
$ cargo run --release -- path/to/qr.png
# ... a big QR code
```

5. Scan the QR code with an open HOTP app like [FreeOTP](https://freeotp.github.io/).
