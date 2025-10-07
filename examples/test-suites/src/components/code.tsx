import { StyleSheet, Text, View } from 'react-native';

export function Code({ code }: { code: string }) {
  return (
    <View style={styles.codeContainer}>
      <Text style={styles.codeText}>{code}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  codeContainer: {
    backgroundColor: '#F8F9FA',
    borderRadius: 8,
    padding: 12,
    borderWidth: 1,
    borderColor: '#E9ECEF',
  },
  codeText: {
    fontFamily: 'monospace',
    fontSize: 12,
    color: '#495057',
    lineHeight: 16,
  },
});
