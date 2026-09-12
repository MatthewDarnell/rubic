import React from 'react';
import { Box, Button, Paper, Typography } from '@mui/material';
import ReportProblemOutlinedIcon from '@mui/icons-material/ReportProblemOutlined';

// Keeps one misbehaving section from blanking the whole window.
export default class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error) {
    return { error };
  }

  componentDidCatch(error, info) {
    console.error(`[${this.props.name || 'section'}]`, error, info?.componentStack);
  }

  componentDidUpdate(prevProps) {
    if (this.state.error && prevProps.resetKey !== this.props.resetKey) {
      this.setState({ error: null });
    }
  }

  render() {
    if (!this.state.error) return this.props.children;
    return (
      <Paper variant='outlined' sx={{ p: 4, textAlign: 'center', borderColor: 'error.main' }}>
        <ReportProblemOutlinedIcon color='error' sx={{ fontSize: 40, mb: 1 }} />
        <Typography variant='h6' sx={{ mb: 0.5 }}>
          Something went wrong in {this.props.name || 'this section'}
        </Typography>
        <Typography variant='body2' color='text.secondary' sx={{ mb: 2 }}>
          The rest of the wallet keeps working. Details are in the console.
        </Typography>
        <Box component='pre' sx={{ fontSize: '0.72rem', color: 'text.secondary', textAlign: 'left', overflow: 'auto', maxHeight: 120, mb: 2 }}>
          {String(this.state.error?.message || this.state.error)}
        </Box>
        <Button variant='outlined' onClick={() => this.setState({ error: null })}>
          Reload section
        </Button>
      </Paper>
    );
  }
}
