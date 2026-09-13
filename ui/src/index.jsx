import React from 'react';
import ReactDOM from 'react-dom/client';
import MainView from './MainView';
import { ToastProvider } from './components/ToastProvider';

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(
  <ToastProvider>
    <MainView />
  </ToastProvider>
);
