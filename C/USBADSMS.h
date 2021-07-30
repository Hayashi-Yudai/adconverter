//-----------------------------------------------
//TUSB-1612ADSM-S�p�h���C�o�A�N�Z�X�pAPI
//Visual C++/CLI �p���W���[��
//
//2014_02_04�@������Ѓ^�[�g���H��
//Copyright (C) 2013 Turtle Industry Co.,Ltd.
//-----------------------------------------------

#ifndef USBADSMS_h
#define USBADSMS_h

using namespace System;
using namespace System::Runtime::InteropServices;

// �A����荞�ݐݒ�p�\���̐錾
[StructLayout(LayoutKind::Sequential)] value struct adsms_setting
//struct adsms_setting
{
    unsigned char ChLen; //�`�����l���� 1-16
    [MarshalAs(UnmanagedType::ByValArray, SizeConst = 16)]
	array<unsigned char> ^ ChPattern;
    [MarshalAs(UnmanagedType::ByValArray, SizeConst = 16)]
	array<unsigned char> ^ ChPatternRange;
    unsigned char TriggerType; //�g���K��ށ@1:�\�t�g�@2:�O���f�B�W�^������� 3:�O���f�B�W�^�������� 4:����G�b�W 5:�����G�b�W�@6:������x���@7:�������x��
    short TriggerLevel; //�G�b�W�⃌�x����臒l
    unsigned char TriggerCh; //�G�b�W�⃌�x�������o����`�����l���ʒu(�p�^�[���o�b�t�@�̃|�C���^)
    unsigned char ClockSel; //0:�����N���b�N�@1:�O���N���b�N
    int SamplingClock; //�����N���b�N�Ԋu 10�`16,777,215[uS]
    int PreTriggerLen; //�v���g���K�� 0�`10,000,000
    int TotalLen; //�S�捞�f�[�^�o�b�t�@ 1�`10,000,000
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
            return "����I�����܂���";
        case 1:
            return "ID�ԍ����s���ł�";
        case 2:
            return "�h���C�o���C���X�g�[������Ă��܂���";
        case 3:
            return "���łɃf�o�C�X�̓I�[�v������Ă��܂�";
        case 4:
            return "�ڑ�����Ă���䐔���������܂�";
        case 5:
            return "�I�[�v���ł��܂���ł���";
        case 6:
            return "�f�o�C�X���݂���܂���";
        case 7:
            return "�I�[�v������Ă��܂���";
        case 8:
            return "�p�����[�^�G���[";
        case 9:
            return "USB�ʐM�G���[�ł�";
        default:
            return "�s���ȃG���[�ł�";
    }
}

#endif
