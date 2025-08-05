import { useEffect, useState } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  ScrollView,
  Image,
} from 'react-native';
import { numeric, string, boolean } from 'basic-module';

export function App() {
  const [numericInput, setNumericInput] = useState('');
  const [stringInput, setStringInput] = useState('');
  const [booleanInput, setBooleanInput] = useState(false);
  const [results, setResults] = useState<{
    numeric: number | null;
    string: string | null;
    boolean: boolean | null;
  }>({
    numeric: null,
    string: null,
    boolean: null,
  });

  useEffect(() => {
    // Test numeric function
    const numValue = parseFloat(numericInput);
    if (!isNaN(numValue)) {
      try {
        setResults(prev => ({ ...prev, numeric: numeric(numValue) }));
      } catch (error) {
        console.warn('Numeric function error:', error);
        setResults(prev => ({ ...prev, numeric: null }));
      }
    } else {
      setResults(prev => ({ ...prev, numeric: null }));
    }
  }, [numericInput]);

  useEffect(() => {
    // Test string function
    if (stringInput.trim()) {
      try {
        setResults(prev => ({ ...prev, string: string(stringInput) }));
      } catch (error) {
        console.warn('String function error:', error);
        setResults(prev => ({ ...prev, string: null }));
      }
    } else {
      setResults(prev => ({ ...prev, string: null }));
    }
  }, [stringInput]);

  useEffect(() => {
    // Test boolean function
    try {
      setResults(prev => ({ ...prev, boolean: boolean(booleanInput) }));
    } catch (error) {
      console.warn('Boolean function error:', error);
      setResults(prev => ({ ...prev, boolean: null }));
    }
  }, [booleanInput]);

  const clear = () => {
    setNumericInput('');
    setStringInput('');
    setBooleanInput(false);
    setResults({ numeric: null, string: null, boolean: null });
  };

  return (
    <ScrollView
      style={styles.container}
      contentContainerStyle={styles.contentContainer}
    >
      {/* React Logo */}
      <View style={styles.logoContainer}>
        <View style={styles.logo}>
          <Image style={styles.logo} source={require('./assets/react.png')} />
        </View>
      </View>

      {/* Title */}
      <Text style={styles.title}>Basic Module Tester</Text>

      {/* Description */}
      <Text style={styles.description}>
        Test numeric, string, boolean, and array functions from basic-module
      </Text>

      {/* Input Section */}
      <View style={styles.inputCard}>
        <View style={styles.inputHeader}>
          <Text style={styles.inputTitle}>Test Inputs</Text>
        </View>

        <TextInput
          style={styles.input}
          placeholder="Numeric input (e.g., 42)"
          value={numericInput}
          onChangeText={text => setNumericInput(text.replace(/[^0-9.-]/g, ''))}
          keyboardType="numeric"
          placeholderTextColor="#999"
        />

        <TextInput
          style={styles.input}
          placeholder="String input (e.g., Hello World)"
          value={stringInput}
          onChangeText={setStringInput}
          placeholderTextColor="#999"
        />

        <View style={styles.booleanContainer}>
          <Text style={styles.booleanLabel}>Boolean input:</Text>
          <TouchableOpacity
            style={[styles.booleanButton, booleanInput && styles.booleanButtonActive]}
            onPress={() => setBooleanInput(!booleanInput)}
          >
            <Text style={[styles.booleanButtonText, booleanInput && styles.booleanButtonTextActive]}>
              {booleanInput ? 'true' : 'false'}
            </Text>
          </TouchableOpacity>
        </View>

        <View style={styles.buttonContainer}>
          <TouchableOpacity style={styles.clearButton} onPress={clear}>
            <Text style={styles.clearButtonText}>Clear All</Text>
          </TouchableOpacity>
        </View>
      </View>

      {/* Test Result Cards */}
      <TestCard
        title="Numeric Function"
        funcName="numeric(number)"
        input={numericInput}
        result={results.numeric}
        color="#10B981"
        type="number"
      />

      <TestCard
        title="String Function"
        funcName="string(string)"
        input={stringInput}
        result={results.string}
        color="#3B82F6"
        type="string"
      />

      <TestCard
        title="Boolean Function"
        funcName="boolean(boolean)"
        input={booleanInput.toString()}
        result={results.boolean}
        color="#8B5CF6"
        type="boolean"
      />

      {/* Footer */}
      <View style={styles.footer}>
        <Text style={styles.footerText}>Basic Module Function Tester</Text>
      </View>
    </ScrollView>
  );
};

function TestCard({
  title,
  funcName,
  input,
  result,
  color,
  type,
}: {
  title: string;
  funcName: string;
  input: string;
  result: any;
  color: string;
  type: string;
}) {
  const formatResult = (value: any, dataType: string) => {
    if (value === null) return '—';
    
    switch (dataType) {
      case 'array':
        return Array.isArray(value) ? `[${value.join(', ')}]` : String(value);
      case 'string':
        return `"${value}"`;
      case 'boolean':
        return String(value);
      case 'number':
        return String(value);
      default:
        return String(value);
    }
  };

  const formatInput = (value: any, dataType: string) => {
    if (!value) return 'No input';
    
    switch (dataType) {
      case 'array':
        return `[${value}]`;
      case 'string':
        return `"${value}"`;
      default:
        return value;
    }
  };

  return (
    <View style={styles.card}>
      <View style={styles.cardContent}>
        <View style={styles.cardLeft}>
          <Text style={styles.cardTitle}>{title}</Text>
          <Text style={styles.cardSubtitle}>{funcName}</Text>
        </View>
        <View style={styles.cardRight}>
          <Text style={[styles.cardResult, { color }]}>
            {formatResult(result, type)}
          </Text>
          <Text style={styles.cardOperation}>
            {input ? `Input: ${formatInput(input, type)}` : 'No input'}
          </Text>
        </View>
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F3F4F6',
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
    aspectRatio: 1,
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
  inputCard: {
    width: '100%',
    backgroundColor: '#DBEAFE',
    borderRadius: 12,
    padding: 16,
    marginTop: 30,
    marginBottom: 20,
  },
  inputHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 15,
  },
  inputTitle: {
    fontSize: 16,
    fontWeight: '500',
    color: '#000',
  },
  input: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 12,
    fontSize: 16,
    marginBottom: 12,
    borderWidth: 1,
    borderColor: '#D1D5DB',
  },
  booleanContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  booleanLabel: {
    fontSize: 16,
    color: '#374151',
    fontWeight: '500',
  },
  booleanButton: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 12,
    paddingHorizontal: 20,
    borderWidth: 1,
    borderColor: '#D1D5DB',
  },
  booleanButtonActive: {
    backgroundColor: '#10B981',
    borderColor: '#10B981',
  },
  booleanButtonText: {
    color: '#374151',
    fontSize: 16,
    fontWeight: '500',
  },
  booleanButtonTextActive: {
    color: '#FFF',
  },
  buttonContainer: {
    flexDirection: 'row',
    gap: 10,
  },
  clearButton: {
    flex: 1,
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 12,
    alignItems: 'center',
    paddingHorizontal: 20,
    borderWidth: 1,
    borderColor: '#D1D5DB',
  },
  clearButtonText: {
    color: '#374151',
    fontSize: 16,
    fontWeight: '500',
  },
  card: {
    width: '100%',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 1,
    },
    shadowOpacity: 0.1,
    shadowRadius: 2,
    elevation: 2,
    borderWidth: 1,
    borderColor: '#E5E7EB',
  },
  cardContent: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  cardLeft: {
    flex: 1,
  },
  cardTitle: {
    fontSize: 18,
    fontWeight: '500',
    color: '#000',
    marginBottom: 4,
  },
  cardSubtitle: {
    fontSize: 14,
    color: '#6B7280',
  },
  cardRight: {
    alignItems: 'flex-end',
  },
  cardResult: {
    fontSize: 24,
    fontWeight: 'bold',
    marginBottom: 4,
  },
  cardOperation: {
    fontSize: 12,
    color: '#6B7280',
  },
  footer: {
    marginTop: 20,
    paddingHorizontal: 20,
  },
  footerText: {
    fontSize: 14,
    color: '#6B7280',
    textAlign: 'center',
    fontFamily: 'monospace',
  },
});
