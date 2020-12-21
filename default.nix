let pkgs = import <nixpkgs> {};
    rustPlatform = pkgs.callPackage ./rust-platform-nightly.nix {};
    qt = pkgs.qt515;
declarativeWidgets = pkgs.stdenv.mkDerivation {
  name = "DeclarativeWidgets";
  src = ./3p/DeclarativeWidgets;

  buildInputs = [ qt.qtdeclarative pkgs.breakpointHook ];
  nativeBuildInputs = [ qt.qmake ];

  prePhases = [ "qmakeFixupPhase" ];
  qmakeFixupPhase = ''
  mkdir tmpout
  tmpout=$(realpath tmpout)
  qmakeFlags+=("declarativewidgets.pro")

  export INSTALL_ROOT=$tmpout
'';
  postInstall = ''
  dir=$(find $tmpout -iname 'QtWidgets')
  mv $dir $out/
'';
};
in (rustPlatform "2020-09-10").buildRustPackage {
  name = "toucan-ui";
  src = ./.;

  buildInputs = [ declarativeWidgets qt.qtbase qt.qtdeclarative
                pkgs.libGL ];

  qtWrapperArgs = [ "--prefix QML2_IMPORT_PATH : ${declarativeWidgets}" ];
  nativeBuildInputs = [ qt.qmake qt.wrapQtAppsHook ];
  
  cargoSha256 = "1rnj0nx3zia3w4wbgsssj97jkrbqfgp7mvrj00nxrnq39j8k6dw9";

  passthru = {
    widgets = declarativeWidgets;
  };
}
