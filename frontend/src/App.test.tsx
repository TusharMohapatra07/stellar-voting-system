import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import App from './App';

describe('App', () => {
  it('renders the app title', () => {
    render(<App />);
    expect(screen.getByText('Stellar Voting DApp')).toBeInTheDocument();
  });

  it('renders placeholder message', () => {
    render(<App />);
    expect(screen.getByText(/Frontend application will be implemented/i)).toBeInTheDocument();
  });
});
