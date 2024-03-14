FROM archlinux

RUN pacman --noconfirm -Sy yazi ffmpegthumbnailer unarchiver jq poppler fd ripgrep fzf zoxide

RUN pacman --noconfirm -Sy vi

ENTRYPOINT yazi

CMD /
