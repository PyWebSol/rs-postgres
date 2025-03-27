#! /bin/bash

ask() {
    echo "$1"
    printf "Choose answer: "
}

ask "Do you want to install Rs-Postgres? (y/n)"
read -r install_rs_postgres
if [ "$install_rs_postgres" = "y" ]; then
    echo "Installing Rs-Postgres..."
else
    echo "Exiting."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    ask "Cargo is not installed. Do you want to install it? (y/n)"
    read -r install_cargo
    if [ "$install_cargo" = "y" ]; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    else
        echo "Cargo is not installed. Exiting."
        exit 1
    fi
fi

if ! command -v cargo &> /dev/null; then
    echo "Cargo is not installed."
    exit 1
fi

if [ ! -d ~/.local/share/applications/ ]; then
    echo "~/.local/share/applications/ does not exist."
    exit 1
fi

if [ ! -d ~/.local/bin/ ]; then
    echo "~/.local/bin/ does not exist."
    exit 1
fi

echo "Building Rs-Postgres and installing..."

cargo build --release

mkdir -p ~/.local/share/rs-postgres/
cp target/release/rs-postgres ~/.local/bin/rs-postgres
cp assets/logo.png ~/.local/share/rs-postgres/logo.png

echo "Creating desktop entry..."

desktop_content=$(cat <<EOF
[Desktop Entry]
Name=Rs-Postgres
Exec=$HOME/.local/bin/rs-postgres
Icon=$HOME/.local/share/rs-postgres/logo.png
Type=Application
Description=Rs-Postgres is a fast and lightweight Rust-based PostgreSQL client with GUI.
EOF
)

echo "$desktop_content" > ~/.local/share/applications/rs-postgres.desktop

ask "Do you want to add ~/.local/bin to PATH? (y/n)"
read -r add_to_path

if [ "$add_to_path" = "y" ]; then
    echo "Adding ~/.local/bin to PATH for all shells..."
    echo ""

    is_any_updated=false

    if [ -f "$HOME/.bashrc" ]; then
        grep -qxF 'export PATH="$HOME/.local/bin:$PATH"' "$HOME/.bashrc" || echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
        echo "Updated .bashrc"
        is_any_updated=true
    fi
    if [ -f "$HOME/.zshrc" ]; then
        grep -qxF 'export PATH="$HOME/.local/bin:$PATH"' "$HOME/.zshrc" || echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
        echo "Updated .zshrc"
        is_any_updated=true
    fi
    if [ -f "$HOME/.profile" ]; then
        grep -qxF 'export PATH="$HOME/.local/bin:$PATH"' "$HOME/.profile" || echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.profile"
        echo "Updated .profile"
        is_any_updated=true
    fi
    if [ -d "$HOME/.config/fish" ]; then
        mkdir -p "$HOME/.config/fish"
        echo 'set -U fish_user_paths $HOME/.local/bin $fish_user_paths' > "$HOME/.config/fish/conf.d/rs-postgres.fish"
        echo "Updated Fish configuration"
        is_any_updated=true
    fi

    if [ "$is_any_updated" = true ]; then
        echo ""
    fi
fi

echo "Rs-Postgres is successfully installed."

if [ "$add_to_path" = "y" ]; then
    echo "Please restart your terminal or start a new shell session for changes to take effect."
fi
