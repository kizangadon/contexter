import { useState } from 'react';
import { Send } from 'lucide-react';
import { PageHeader } from '@/components/layout/PageHeader';
import { Button } from '@/components/ui/Button';

export function PlaygroundPage() {
  const [input, setInput] = useState('');
  const [response, setResponse] = useState('');

  const handleSubmit = () => {
    if (!input.trim()) return;
    setResponse(`You entered: "${input}"`);
    setInput('');
  };

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Playground" />

      <div className="flex flex-col gap-4 rounded-lg border border-border bg-surface p-6">
        <label htmlFor="playground-input" className="text-sm font-medium text-text-primary">
          Enter your message
        </label>
        <textarea
          id="playground-input"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Type something..."
          rows={6}
          className="w-full resize-none rounded-md border border-border bg-bg-primary px-4 py-3 text-sm text-text-primary placeholder:text-text-tertiary focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
        />
        <div className="flex justify-end">
          <Button variant="primary" onClick={handleSubmit} disabled={!input.trim()}>
            <Send className="h-4 w-4" />
            Submit
          </Button>
        </div>
      </div>

      {/* Response display */}
      <section className="flex flex-col gap-4">
        <h2 className="text-lg font-semibold text-text-primary">Response</h2>
        <div className="min-h-[120px] rounded-lg border border-border bg-surface p-4">
          {response ? (
            <p className="text-sm text-text-primary">{response}</p>
          ) : (
            <p className="text-sm italic text-text-tertiary">
              Submit a message to see the response here.
            </p>
          )}
        </div>
      </section>
    </div>
  );
}
