{ fetchurl
, stdenv
, leac
, jdk8
}: let
  junit = fetchurl {
    url = "https://repo1.maven.org/maven2/junit/junit/4.13.2/junit-4.13.2.jar";
    hash = "sha256-jklbY0Rp1k+4rPo0laBly6zIoP/1XOHjEAe+TBbcV9M=";
  };
  hamcrest = fetchurl {
    url = "https://repo1.maven.org/maven2/org/hamcrest/hamcrest-core/1.3/hamcrest-core-1.3.jar";
    hash = "sha256-Zv3vkelzk0jfeglqo4SlaF9Oh1WEzOiThqekclHE2Ok=";
  };
in stdenv.mkDerivation {
  pname = "lea-test";
  version = "0.1";

  buildInputs = [ leac jdk8 ];
  src = ./test;

  buildPhase = ''
    leac tests.lea

    cp ${junit} junit.jar
    cp ${hamcrest} hamcrest.jar
    javac -cp ".:junit.jar:hamcrest.jar" Main.java
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp Main.class Tests.class junit.jar hamcrest.jar $out/bin
    echo "java -cp \"$out/bin:$out/bin/junit.jar:$out/bin/hamcrest.jar\" org.junit.runner.JUnitCore Main" > $out/bin/lea-test
    chmod +x $out/bin/lea-test
  '';
}
