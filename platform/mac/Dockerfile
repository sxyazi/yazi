ARG OWNSDK="11.3"
ARG MACMIN="11.0"
# For Apple Silicon
ARG ARCH="aarch64"
# For Intel
# ARG ARCH="x86_64"

FROM rust:latest AS rustbuilder

ARG OWNSDK
ARG MACMIN
ARG ARCH

RUN apt-get update \
 && apt-get install -y --no-install-recommends build-essential ca-certificates clang cmake curl git llvm lld make pkg-config python3 xz-utils \
 && git clone --depth 1 https://github.com/tpoechtrager/osxcross.git /osxcross \
 && rm -rf /var/lib/apt/lists/* \
;
COPY platform/mac/*.sdk.tar.xz /mac/

RUN if [ "$OWNSDK" = "11.3" ] ; then \
        curl -L -o /osxcross/tarballs/MacOSX11.3.sdk.tar.xz https://github.com/phracker/MacOSX-SDKs/releases/download/11.3/MacOSX11.3.sdk.tar.xz ;\
    else \
        cp /mac/MacOSX${OWNSDK}.sdk.tar.xz /osxcross/tarballs/ ;\
    fi \
 && UNATTENDED=1 OSX_VERSION_MIN=${MACMIN} /osxcross/build.sh \
 && echo rm -rf /osxcross/tarballs/* \
;
RUN rustup target add ${ARCH}-apple-darwin \
;

FROM rustbuilder AS macyazi

ARG MACMIN
ARG ARCH
ARG OWNSDK

WORKDIR /yazi
COPY . .
RUN export PATH=/osxcross/target/bin:$PATH \
 && export SDKROOT=/osxcross/target/SDK/MacOSX${OWNSDK}.sdk \
 && export MACOSX_DEPLOYMENT_TARGET=${MACMIN} \
 && if [ "$ARCH" = "aarch64" ] ; then CSTUFF="oa64" ; else CSTUFF="o64" ; fi \
 && ARCH_KEY="${ARCH}_apple_darwin" \
 && ARCH_UPPER="$( echo "$ARCH_KEY" | tr a-z A-Z )" \
 && eval "export CC_${ARCH_KEY}=${CSTUFF}-clang" \
 && eval "export CXX_${ARCH_KEY}=${CSTUFF}-clang++" \
 && eval "export CARGO_TARGET_${ARCH_UPPER}_LINKER=${CSTUFF}-clang" \
 && eval "export AR_${ARCH_KEY}=llvm-ar" \
 && eval "export RANLIB_${ARCH_KEY}=llvm-ranlib" \
 && cargo build --locked --release --target ${ARCH}-apple-darwin \
;

FROM scratch

ARG ARCH

COPY --from=macyazi /yazi/target/${ARCH}-apple-darwin/release/ya /yazi/target/${ARCH}-apple-darwin/release/yazi /
