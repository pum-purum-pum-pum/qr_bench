New qr lib is able to handle of of these! (it's `quirc` -- pretty mature C library) 
However you image is not handled. I think despite of it being small it's very noisy(pixels on the screen) and it's also not working with `pyzbar`.

I updated the repo so you can:
```
git pull
cargo clean
cargo run --release
```

The images that I sent you (and this library handles) are encoding "lxsdsds-1:david:cowman:david@gojump-america.com:OS:i:0001110"
and ecc_level is "L"

Also I'm used another lib for generation codes. I can add this feature to the final code.