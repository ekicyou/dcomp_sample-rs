# Requirements Document

## Project Description (Input)
マルチモニター間移動時のちらつき修正。ウィンドウを異なるDPIのモニター間で移動する際に、ウィンドウサイズが大きくなったり小さくなったりとちらつく現象が発生している。また、ウィンドウ移動時のウィンドウサイズ再計算に狂いが出ている可能性がある。wndproc内でworld tickが呼ばれた場合にWindowPos呼び出しを抑制する方が良いかもしれない。調査および修正を行う。

## Requirements
<!-- Will be generated in /kiro-spec-requirements phase -->

