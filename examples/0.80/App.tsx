import { useState } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  ScrollView,
  Image,
} from 'react-native';
import { TEST_SUITES } from 'test-suites';

export function App() {
  const [testResults, setTestResults] = useState<Array<{
    label: string;
    description?: string;
    result: any;
    error?: string;
  }>>([]);
  const [isRunning, setIsRunning] = useState(false);

  const runAllTests = async () => {
    setIsRunning(true);
    setTestResults([]);

    const results = [];
    for (const test of TEST_SUITES) {
      try {
        const result = await test.action();
        results.push({
          label: test.label,
          description: test.description,
          result: result,
        });
      } catch (error) {
        results.push({
          label: test.label,
          description: test.description,
          result: null,
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }

    setTestResults(results);
    setIsRunning(false);
  };

  return (
    <ScrollView
      style={styles.container}
      contentContainerStyle={styles.contentContainer}
    >
      {/* Logo */}
      <View style={styles.logoContainer}>
        <View style={styles.logo}>
          <Image style={styles.logo} source={require('./assets/logo.png')} />
        </View>
      </View>

      {/* Title */}
      <Text style={styles.title}>Test Suite Runner</Text>

      {/* Description */}
      <Text style={styles.description}>
        Run all test suites and view results
      </Text>

      {/* Run Test Button */}
      <View style={styles.buttonCard}>
        <TouchableOpacity
          style={[styles.runButton, isRunning && styles.runButtonDisabled]}
          onPress={runAllTests}
          disabled={isRunning}
        >
          <Text style={styles.runButtonText}>
            {isRunning ? 'Running Tests...' : 'Run All Tests'}
          </Text>
        </TouchableOpacity>
      </View>

      {/* Test Results */}
      {testResults.map((testResult, index) => (
        <TestResultCard
          key={index}
          label={testResult.label}
          description={testResult.description}
          result={testResult.result}
          error={testResult.error}
        />
      ))}
    </ScrollView>
  );
}

function Code({ children }: { children: string }) {
  return (
    <View style={styles.codeContainer}>
      <Text style={styles.codeText}>{children}</Text>
    </View>
  );
}

function TestResultCard({
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
        <Text style={[styles.cardStatus, { color: statusColor }]}>
          {isSuccess ? 'Passed' : 'Error'}
        </Text>
      </View>

      {description ? (
        <View style={styles.cardDescription}>
          <Text style={styles.cardDescriptionText}>{description}</Text>
        </View>
      ) : null}

      <View style={styles.cardBody}>
        {error ? <Text style={styles.cardError}>{error}</Text> : <Code>{formattedResult}</Code>}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  contentContainer: {
    alignItems: 'center',
    paddingHorizontal: 20,
    paddingTop: 60,
    paddingBottom: 40,
  },
  logoContainer: {
    marginTop: 64,
    marginBottom: 30,
  },
  logo: {
    height: 80,
    resizeMode: 'contain',
    marginBottom: 24,
  },
  title: {
    fontSize: 28,
    fontWeight: '300',
    color: '#000',
    marginBottom: 10,
    textAlign: 'center',
  },
  description: {
    fontSize: 16,
    color: '#6B7280',
    marginBottom: 5,
    textAlign: 'center',
  },
  buttonCard: {
    width: '100%',
    marginTop: 30,
    marginBottom: 20,
  },
  runButton: {
    width: '100%',
    backgroundColor: '#387ca0',
    borderRadius: 8,
    padding: 16,
    alignItems: 'center',
  },
  runButtonDisabled: {
    backgroundColor: '#9CA3AF',
  },
  runButtonText: {
    color: '#FFF',
    fontSize: 16,
    fontWeight: '600',
  },
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
