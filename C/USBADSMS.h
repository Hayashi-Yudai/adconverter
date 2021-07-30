//-----------------------------------------------
//TUSB-1612ADSM-S用ドライバアクセス用API
//Visual C++/CLI 用モジュール
//
//2014_02_04　株式会社タートル工業
//Copyright (C) 2013 Turtle Industry Co.,Ltd.
//-----------------------------------------------

#ifndef USBADSMS_h
#define USBADSMS_h

using namespace System;
using namespace System::Runtime::InteropServices;

// 連続取り込み設定用構造体宣言
[StructLayout(LayoutKind::Sequential)] value struct adsms_setting
//struct adsms_setting
{
    unsigned char ChLen; //チャンネル長 1-16
    [MarshalAs(UnmanagedType::ByValArray, SizeConst = 16)]
	array<unsigned char> ^ ChPattern;
    [MarshalAs(UnmanagedType::ByValArray, SizeConst = 16)]
	array<unsigned char> ^ ChPatternRange;
    unsigned char TriggerType; //トリガ種類　1:ソフト　2:外部ディジタル立上り 3:外部ディジタル立下り 4:立上エッジ 5:立下エッジ　6:上限レベル　7:下限レベル
    short TriggerLevel; //エッジやレベルの閾値
    unsigned char TriggerCh; //エッジやレベルを検出するチャンネル位置(パターンバッファのポインタ)
    unsigned char ClockSel; //0:内部クロック　1:外部クロック
    int SamplingClock; //内部クロック間隔 10～16,777,215[uS]
    int PreTriggerLen; //プレトリガ長 0～10,000,000
    int TotalLen; //全取込データバッファ 1～10,000,000
};

[DllImport("TUSADSMS.DLL", CallingConvention = CallingConvention::Cdecl)]
extern short Tusbadsms_Device_Open(short id);
[DllImport("TUSADSMS.DLL", CallingConvention = CallingConvention::Cdecl)]
extern void Tusbadsms_Device_Close(short id);
[DllImport("TUSADSMS.DLL", CallingConvention = CallingConvention::Cdecl)]
extern short Tusbadsms_Pio_Write(short id, unsigned char dat);
[DllImport("TUSADSMS.DLL", CallingConvention = CallingConvention::Cdecl)]
extern short Tusbadsms_Pio_Read(short id, unsigned char *dat);
[DllImport("TUSADSMS.DLL", CallingConvention = CallingConvention::Cdecl)]
extern short Tusbadsms_Single_Sample(short id, unsigned char ch, unsigned char range, short *dat);
[DllImport("TUSADSMS.DLL", CallingConvention = CallingConvention::Cdecl)]
extern short Tusbadsms_Status_Read(short id, unsigned char *status, unsigned char *ovf, int *leng);
[DllImport("TUSADSMS.DLL", CallingConvention = CallingConvention::Cdecl)]
extern short Tusbadsms_Memory_Read(short id, [Out] short Data[], int *leng);
[DllImport("TUSADSMS.DLL", CallingConvention = CallingConvention::Cdecl)]
extern short Tusbadsms_Sampling_Stop(short id);
[DllImport("TUSADSMS.DLL", CallingConvention = CallingConvention::Cdecl)]
extern short Tusbadsms_Memory_Clear(short id);
[DllImport("TUSADSMS.DLL", CallingConvention = CallingConvention::Cdecl)]
extern short Tusbadsms_Sampling_Trigger(short id);
[DllImport("TUSADSMS.DLL", CallingConvention = CallingConvention::Cdecl)]
extern short Tusbadsms_Sample_Start(short id, [In] adsms_setting smplset);

static String^ Tusbadsms_GetErrMessage(short retcode)
{
    switch (retcode)
    {
        case 0:
            return "正常終了しました";
        case 1:
            return "ID番号が不正です";
        case 2:
            return "ドライバがインストールされていません";
        case 3:
            return "すでにデバイスはオープンされています";
        case 4:
            return "接続されている台数が多すぎます";
        case 5:
            return "オープンできませんでした";
        case 6:
            return "デバイスがみつかりません";
        case 7:
            return "オープンされていません";
        case 8:
            return "パラメータエラー";
        case 9:
            return "USB通信エラーです";
        default:
            return "不明なエラーです";
    }
}

#endif
