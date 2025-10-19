# jelly-fpga-loader


jelly-fpga-client-rs を使った FPGA ユーティリティです。

基本的にまだ KV-260 でしか動作確認しておりませんので、KV260 専用ツールと考えてください。


## Usage


### bitstream の単純ダウンロード

```bash
jelly-fpga-loader bitdownload <bitstream file> -ip <FPGA server IP address>
```

を指定すると、指定した bitstream ファイルを FPGA に書き込みます。

オプションで -ip を指定しなかった場合はローカル(127.0.0.1:8051) に接続します。


### DeviceTree Overlay の適用

```bash
jelly-fpga-loader overlay <dtbo file> -bit <bitstream file> -ip <FPGA server IP address>
```

や

```bash
jelly-fpga-loader overlay <dtbo file> -bin <bin file> -ip <FPGA server IP address>
```


を指定すると、指定した bitstream ファイルを firmware に転送した後に、DeviceTree Overlay を適用します。

bitstream は 転送後 bin ファイルに変換され xxxx.bit という名前の場合は xxxx.bit.bin と言う名前に変換されて扱われます。 -bin オプションで指定した場合はそのまま転送されます。

DeviceTree が bitstream を参照していた場合に -bit オプション や -bin オプションで指定することで合わせて転送できます。

<dtbo file> に指定したファイルの拡張子が .dtbo または .dtb の場合はそのまま転送し、.dts の場合は内部で dtc を呼び出してコンパイルしてから転送します。

オプションで -ip を指定しなかった場合はローカル(127.0.0.1:8051) に接続します。


### アクセラレータの登録

```bash
jelly-fpga-loader register-accel <accel name> <dtbo file> <bitstream file> -json <json file> -ip <FPGA server IP address>
```

Xilinx の xmutil コマンドや dfx-mgr-client コマンドで使用するアクセラレータパッケージを登録します。

各ファイルは内部で個別に firmware に転送された後に登録されます。

<bitstream file> は、拡張子が .bin であればそのまま、 .bit であれば .bin に変換して転送されます。

<dtbo file> に指定したファイルの拡張子が .dtbo または .dtb の場合はそのまま転送し、.dts の場合は内部で dtc を呼び出してコンパイルしてから転送します。

### アクセラレータの登録解除

```bash
jelly-fpga-loader unregister-accel <accel name> -ip <FPGA server IP address>
```

を指定すると、指定したアクセラレータパッケージを登録解除します。

オプションで -ip を指定しなかった場合はローカル(127.0.0.1:8051) に接続します。

### アクセラレータのロード

```bash
jelly-fpga-loader load <accel name> -ip <FPGA server IP address>
```

を指定すると、指定したアクセラレータパッケージをロードします。

オプションで -ip を指定しなかった場合はローカル(127.0.0.1:8051) に接続します。

### アクセラレータのアンロード

```bash
jelly-fpga-loader unload <slot> -ip <FPGA server IP address>
```

を指定すると、指定したスロットからアクセラレータパッケージをアンロードします。

スロット番号を省略すると 0 番スロットをアンロードします。

オプションで -ip を指定しなかった場合はローカル(127.0.0.1:8051) に接続します。


### dts ファイルの dtb ファイルへの変換

```bash
jelly-fpga-loader dts2dtbo <dts file> <dtbo file>
```

を指定すると、サーバーに dts ファイルを送って dtbo ファイルに変換します。

オプションで -ip を指定しなかった場合はローカル(127.0.0.1:8051) に接続します。





