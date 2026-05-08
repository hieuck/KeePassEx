/**
 * OTP Setup Modal — shared component for adding OTP to an entry
 * Supports manual entry and QR code scanning
 */
import { useState } from 'react';
import {
  View, Text, TextInput, TouchableOpacity,
  StyleSheet, Modal, ScrollView,
} from 'react-native';
import { tokens } from '../tokens';

export interface OtpSetupData {
  secret: string;
  issuer: string;
  account: string;
  algorithm: 'SHA1' | 'SHA256' | 'SHA512';
  digits: 6 | 8;
  period: 30 | 60;
}

interface OtpSetupModalProps {
  visible: boolean;
  onClose: () => void;
  onSave: (data: OtpSetupData) => void;
  onScanQr?: () => void;
  initialData?: Partial<OtpSetupData>;
  locale?: 'en' | 'vi';
}

const LABELS = {
  en: {
    title: 'Set Up One-Time Password',
    secret: 'Secret Key',
    secretPlaceholder: 'JBSWY3DPEHPK3PXP',
    issuer: 'Issuer (optional)',
    issuerPlaceholder: 'GitHub',
    account: 'Account (optional)',
    accountPlaceholder: 'user@example.com',
    algorithm: 'Algorithm',
    digits: 'Digits',
    period: 'Period (seconds)',
    scanQr: 'Scan QR Code',
    save: 'Save OTP',
    cancel: 'Cancel',
    invalidSecret: 'Invalid secret key. Must be base32 encoded.',
  },
  vi: {
    title: 'Thiết lập mật khẩu một lần',
    secret: 'Khóa bí mật',
    secretPlaceholder: 'JBSWY3DPEHPK3PXP',
    issuer: 'Nhà phát hành (tùy chọn)',
    issuerPlaceholder: 'GitHub',
    account: 'Tài khoản (tùy chọn)',
    accountPlaceholder: 'user@example.com',
    algorithm: 'Thuật toán',
    digits: 'Số chữ số',
    period: 'Chu kỳ (giây)',
    scanQr: 'Quét mã QR',
    save: 'Lưu OTP',
    cancel: 'Hủy',
    invalidSecret: 'Khóa bí mật không hợp lệ. Phải là base32.',
  },
};

function isValidBase32(secret: string): boolean {
  const cleaned = secret.toUpperCase().replace(/\s|-/g, '');
  return /^[A-Z2-7]+=*$/.test(cleaned) && cleaned.length >= 16;
}

export function OtpSetupModal({
  visible,
  onClose,
  onSave,
  onScanQr,
  initialData,
  locale = 'en',
}: OtpSetupModalProps) {
  const L = LABELS[locale];

  const [secret, setSecret] = useState(initialData?.secret ?? '');
  const [issuer, setIssuer] = useState(initialData?.issuer ?? '');
  const [account, setAccount] = useState(initialData?.account ?? '');
  const [algorithm, setAlgorithm] = useState<'SHA1' | 'SHA256' | 'SHA512'>(
    initialData?.algorithm ?? 'SHA1'
  );
  const [digits, setDigits] = useState<6 | 8>(initialData?.digits ?? 6);
  const [period, setPeriod] = useState<30 | 60>(initialData?.period ?? 30);
  const [secretError, setSecretError] = useState('');

  const handleSave = () => {
    const cleaned = secret.trim().toUpperCase().replace(/\s|-/g, '');
    if (!isValidBase32(cleaned)) {
      setSecretError(L.invalidSecret);
      return;
    }
    setSecretError('');
    onSave({ secret: cleaned, issuer, account, algorithm, digits, period });
  };

  const inputStyle = [styles.input];

  return (
    <Modal
      visible={visible}
      animationType="slide"
      presentationStyle="pageSheet"
      onRequestClose={onClose}
    >
      <View style={styles.container}>
        {/* Header */}
        <View style={styles.header}>
          <TouchableOpacity onPress={onClose} accessibilityRole="button" accessibilityLabel={L.cancel}>
            <Text style={styles.cancelBtn}>{L.cancel}</Text>
          </TouchableOpacity>
          <Text style={styles.title}>{L.title}</Text>
          <TouchableOpacity
            onPress={handleSave}
            disabled={!secret.trim()}
            accessibilityRole="button"
            accessibilityLabel={L.save}
          >
            <Text style={[styles.saveBtn, !secret.trim() && styles.saveBtnDisabled]}>
              {L.save}
            </Text>
          </TouchableOpacity>
        </View>

        <ScrollView style={styles.content} contentContainerStyle={styles.contentInner}>
          {/* QR scan button */}
          {onScanQr && (
            <TouchableOpacity
              style={styles.qrButton}
              onPress={onScanQr}
              accessibilityRole="button"
              accessibilityLabel={L.scanQr}
            >
              <Text style={styles.qrIcon}>📷</Text>
              <Text style={styles.qrText}>{L.scanQr}</Text>
            </TouchableOpacity>
          )}

          {/* Secret */}
          <View style={styles.field}>
            <Text style={styles.label}>{L.secret} *</Text>
            <TextInput
              style={[inputStyle, secretError ? styles.inputError : null]}
              value={secret}
              onChangeText={v => { setSecret(v); setSecretError(''); }}
              placeholder={L.secretPlaceholder}
              placeholderTextColor={tokens.color.gray400}
              autoCapitalize="characters"
              autoCorrect={false}
              accessibilityLabel={L.secret}
            />
            {secretError ? (
              <Text style={styles.errorText} accessibilityRole="alert">{secretError}</Text>
            ) : null}
          </View>

          {/* Issuer */}
          <View style={styles.field}>
            <Text style={styles.label}>{L.issuer}</Text>
            <TextInput
              style={inputStyle}
              value={issuer}
              onChangeText={setIssuer}
              placeholder={L.issuerPlaceholder}
              placeholderTextColor={tokens.color.gray400}
              accessibilityLabel={L.issuer}
            />
          </View>

          {/* Account */}
          <View style={styles.field}>
            <Text style={styles.label}>{L.account}</Text>
            <TextInput
              style={inputStyle}
              value={account}
              onChangeText={setAccount}
              placeholder={L.accountPlaceholder}
              placeholderTextColor={tokens.color.gray400}
              keyboardType="email-address"
              autoCapitalize="none"
              accessibilityLabel={L.account}
            />
          </View>

          {/* Algorithm */}
          <View style={styles.field}>
            <Text style={styles.label}>{L.algorithm}</Text>
            <View style={styles.segmented} role="radiogroup">
              {(['SHA1', 'SHA256', 'SHA512'] as const).map(alg => (
                <TouchableOpacity
                  key={alg}
                  style={[styles.segment, algorithm === alg && styles.segmentActive]}
                  onPress={() => setAlgorithm(alg)}
                  accessibilityRole="radio"
                  accessibilityState={{ checked: algorithm === alg }}
                >
                  <Text style={[styles.segmentText, algorithm === alg && styles.segmentTextActive]}>
                    {alg}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>
          </View>

          {/* Digits */}
          <View style={styles.field}>
            <Text style={styles.label}>{L.digits}</Text>
            <View style={styles.segmented} role="radiogroup">
              {([6, 8] as const).map(d => (
                <TouchableOpacity
                  key={d}
                  style={[styles.segment, digits === d && styles.segmentActive]}
                  onPress={() => setDigits(d)}
                  accessibilityRole="radio"
                  accessibilityState={{ checked: digits === d }}
                >
                  <Text style={[styles.segmentText, digits === d && styles.segmentTextActive]}>
                    {d}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>
          </View>

          {/* Period */}
          <View style={styles.field}>
            <Text style={styles.label}>{L.period}</Text>
            <View style={styles.segmented} role="radiogroup">
              {([30, 60] as const).map(p => (
                <TouchableOpacity
                  key={p}
                  style={[styles.segment, period === p && styles.segmentActive]}
                  onPress={() => setPeriod(p)}
                  accessibilityRole="radio"
                  accessibilityState={{ checked: period === p }}
                >
                  <Text style={[styles.segmentText, period === p && styles.segmentTextActive]}>
                    {p}s
                  </Text>
                </TouchableOpacity>
              ))}
            </View>
          </View>
        </ScrollView>
      </View>
    </Modal>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: tokens.color.white },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: tokens.space.lg,
    paddingVertical: tokens.space.md,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: tokens.color.gray200,
  },
  title: {
    fontSize: tokens.fontSize.lg,
    fontWeight: tokens.fontWeight.semibold,
    color: tokens.color.gray900,
  },
  cancelBtn: {
    fontSize: tokens.fontSize.md,
    color: tokens.color.primary,
  },
  saveBtn: {
    fontSize: tokens.fontSize.md,
    fontWeight: tokens.fontWeight.semibold,
    color: tokens.color.primary,
  },
  saveBtnDisabled: { opacity: 0.4 },
  content: { flex: 1 },
  contentInner: {
    padding: tokens.space.lg,
    gap: tokens.space.lg,
  },
  qrButton: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: tokens.space.sm,
    padding: tokens.space.md,
    backgroundColor: tokens.color.gray100,
    borderRadius: tokens.radius.md,
    borderWidth: 1,
    borderColor: tokens.color.gray200,
    borderStyle: 'dashed',
  },
  qrIcon: { fontSize: 24 },
  qrText: {
    fontSize: tokens.fontSize.md,
    fontWeight: tokens.fontWeight.medium,
    color: tokens.color.primary,
  },
  field: { gap: tokens.space.xs },
  label: {
    fontSize: tokens.fontSize.sm,
    fontWeight: tokens.fontWeight.medium,
    color: tokens.color.gray600,
    textTransform: 'uppercase',
    letterSpacing: 0.5,
  },
  input: {
    borderWidth: 1,
    borderColor: tokens.color.gray300,
    borderRadius: tokens.radius.md,
    paddingHorizontal: tokens.space.md,
    paddingVertical: tokens.space.sm,
    fontSize: tokens.fontSize.md,
    color: tokens.color.gray900,
    backgroundColor: tokens.color.white,
    fontFamily: 'Menlo',
  },
  inputError: { borderColor: tokens.color.danger },
  errorText: {
    fontSize: tokens.fontSize.xs,
    color: tokens.color.danger,
  },
  segmented: {
    flexDirection: 'row',
    backgroundColor: tokens.color.gray100,
    borderRadius: tokens.radius.md,
    padding: 3,
    gap: 2,
  },
  segment: {
    flex: 1,
    paddingVertical: tokens.space.xs,
    borderRadius: tokens.radius.sm,
    alignItems: 'center',
  },
  segmentActive: {
    backgroundColor: tokens.color.white,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.1,
    shadowRadius: 2,
    elevation: 2,
  },
  segmentText: {
    fontSize: tokens.fontSize.sm,
    color: tokens.color.gray500,
    fontWeight: tokens.fontWeight.medium,
  },
  segmentTextActive: {
    color: tokens.color.gray900,
    fontWeight: tokens.fontWeight.semibold,
  },
});
