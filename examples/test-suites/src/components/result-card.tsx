import { StyleSheet, Text, View } from 'react-native';
import { Code } from './code';

export function ResultCard({
  label,
  description,
  result,
  error,
}: {
  label: string;
  description?: string;
  result: any;
  error?: string;
}) {
  const formatResult = (value: any): string => {
    if (value === null) {
      return 'null';
    }

    if (value === undefined) {
      return 'undefined';
    }

    if (Array.isArray(value)) {
      return `[${value.map(formatResult).join(', ')}]`;
    }

    return JSON.stringify(value, null, 4);
  };

  const isSuccess = !error;
  const statusColor = isSuccess ? '#10B981' : '#EF4444';
  const formattedResult = formatResult(result);

  return (
    <View style={styles.card}>
      <View style={styles.cardHeader}>
        <Text style={styles.cardTitle}>{label}</Text>
        <Text style={[styles.cardStatus, { color: statusColor }]}>{isSuccess ? 'Passed' : 'Error'}</Text>
      </View>

      {description ? (
        <View style={styles.cardDescription}>
          <Text style={styles.cardDescriptionText}>{description}</Text>
        </View>
      ) : null}

      <View style={styles.cardBody}>
        {error ? <Text style={styles.cardError}>{error}</Text> : <Code code={formattedResult} />}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  card: {
    width: '100%',
    paddingVertical: 16,
    paddingHorizontal: 8,
    borderBottomWidth: 1,
    borderBottomColor: '#E9ECEF',
  },
  cardHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  cardTitle: {
    fontSize: 18,
    fontWeight: '500',
    color: '#000',
  },
  cardStatus: {
    fontSize: 14,
    fontWeight: '500',
  },
  cardDescription: {
    marginTop: -8,
    marginBottom: 12,
  },
  cardDescriptionText: {
    fontSize: 12,
    color: '#6B7280',
  },
  cardBody: {
    width: '100%',
  },
  cardResult: {
    fontSize: 16,
    color: '#374151',
    fontFamily: 'monospace',
  },
  cardError: {
    fontSize: 14,
    color: '#EF4444',
  },
});
