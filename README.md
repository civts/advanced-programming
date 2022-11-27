🦀 Advanced Programming Course Project 🦀

# Setup

## Dependencies

If you use [nix](https://nixos.org), a flake is here for you. Activate the environment with `direnv allow` and you will get the right versions of everything and a VScode already configured with all the extensions.

Otherwise, you will need the following:
- rustup (v1.25.1)
- cargo (v1.64.0)
- rustfmt (v1.5.1-stable)
- your code editor of choice

## Connect to the VPN
1. Ping `advancedprogramming.disi.unitn.it`. This should fail since you'll need to connect to University of Trento's VPN to access `advancedprogramming.disi.unitn.it`

1.  Open the GlobalProtect VPN client by running `sudo gpclient` (be sure to follow the setup instructions from the [official repo](https://github.com/yuezk/GlobalProtect-openconnect))

1.  Enter as portal address `vpn.icts.unitn.it`

1.  The username is your unitn email without the "studenti" subdomain -e.g., `mario.rossi@unitn.it`-

1.  The password is your unitn password

1.  The VPN will show the status 'Connected'.

1.  Now you should be able to successfully ping `advancedprogramming.disi.unitn.it`

## Add your auth token to be able to fetch from Kellnr

1. Create a file `.env` with the same contents of `.env.sample` (to do so, you can run `cp .env.sample .env`).

1. Add the token you have received by mail from tech.disi@unitn.it in date 20/10/22 in the `.env`.

1. Note that previously active shells/terminal emulators may not pick up automatically the new environment variable and you may need to restart them.

1. `cd` in a crate and run a `cargo update` to confirm you have access to `kellnr`.
