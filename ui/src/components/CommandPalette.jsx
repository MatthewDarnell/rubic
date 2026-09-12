import React, { useEffect, useMemo, useRef, useState } from 'react';
import {
  Box,
  Dialog,
  InputBase,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Typography,
} from '@mui/material';
import SearchIcon from '@mui/icons-material/Search';
import SendIcon from '@mui/icons-material/Send';
import AddIcon from '@mui/icons-material/Add';
import DarkModeIcon from '@mui/icons-material/DarkMode';
import Identicon from './Identicon';
import { NAV } from './AppShell';
import { shortenId } from '../utils/format';

// Ctrl+K: jump to a section or identity, or trigger a common action.
export default function CommandPalette({ open, onClose, identities, labels, onNav, onOpenIdentity, onSend, onCreate, onToggleTheme }) {
  const [query, setQuery] = useState('');
  const [active, setActive] = useState(0);
  const listRef = useRef(null);

  useEffect(() => {
    if (open) {
      setQuery('');
      setActive(0);
    }
  }, [open]);

  const commands = useMemo(() => {
    const q = query.trim().toLowerCase();
    const out = [];
    NAV.forEach((n) => out.push({ key: `nav-${n.key}`, group: 'Go to', label: n.label, Icon: n.icon, run: () => onNav(n.key) }));
    out.push({ key: 'send', group: 'Actions', label: 'Send QU', Icon: SendIcon, run: () => onSend() });
    out.push({ key: 'create', group: 'Actions', label: 'Create new identity', Icon: AddIcon, run: () => onCreate() });
    out.push({ key: 'theme', group: 'Actions', label: 'Toggle light / dark theme', Icon: DarkModeIcon, run: () => onToggleTheme() });
    identities.forEach((i) =>
      out.push({
        key: `id-${i.id}`,
        group: 'Identities',
        label: labels[i.id] || shortenId(i.id, 10, 10),
        sub: labels[i.id] ? i.id : null,
        identity: i.id,
        run: () => onOpenIdentity(i.id),
      })
    );
    if (!q) return out;
    return out.filter((c) => c.label.toLowerCase().includes(q) || (c.sub || '').toLowerCase().includes(q) || (c.identity || '').toLowerCase().includes(q));
  }, [query, identities, labels, onNav, onSend, onCreate, onToggleTheme, onOpenIdentity]);

  useEffect(() => {
    setActive(0);
  }, [query]);

  const run = (c) => {
    onClose();
    c.run();
  };

  const onKeyDown = (e) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setActive((a) => Math.min(commands.length - 1, a + 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setActive((a) => Math.max(0, a - 1));
    } else if (e.key === 'Enter' && commands[active]) {
      e.preventDefault();
      run(commands[active]);
    }
  };

  useEffect(() => {
    listRef.current?.querySelector(`[data-index="${active}"]`)?.scrollIntoView({ block: 'nearest' });
  }, [active]);

  let lastGroup = null;
  return (
    <Dialog open={open} onClose={onClose} maxWidth='sm' fullWidth slotProps={{ paper: { sx: { mt: -20 } } }}>
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, px: 2, py: 1.5, borderBottom: 1, borderColor: 'divider' }}>
        <SearchIcon color='action' />
        <InputBase
          autoFocus
          fullWidth
          placeholder='Jump to a section, identity, or action…'
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={onKeyDown}
          sx={{ fontSize: '1rem' }}
        />
        <Typography variant='caption' color='text.secondary'>
          Esc
        </Typography>
      </Box>
      <List ref={listRef} dense sx={{ maxHeight: 380, overflow: 'auto', py: 1 }}>
        {commands.length === 0 && (
          <Typography variant='body2' color='text.secondary' sx={{ px: 2, py: 2 }}>
            No matches.
          </Typography>
        )}
        {commands.map((c, index) => {
          const header = c.group !== lastGroup ? c.group : null;
          lastGroup = c.group;
          return (
            <React.Fragment key={c.key}>
              {header && (
                <Typography variant='caption' sx={{ px: 2, pt: 1, display: 'block', color: 'text.secondary', fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                  {header}
                </Typography>
              )}
              <ListItemButton data-index={index} selected={index === active} onMouseEnter={() => setActive(index)} onClick={() => run(c)}>
                <ListItemIcon sx={{ minWidth: 36 }}>
                  {c.identity ? <Identicon id={c.identity} size={22} /> : c.Icon ? <c.Icon fontSize='small' /> : null}
                </ListItemIcon>
                <ListItemText
                  primary={c.label}
                  secondary={c.sub}
                  slotProps={{ secondary: { className: 'mono', sx: { fontSize: '0.72rem' } } }}
                />
              </ListItemButton>
            </React.Fragment>
          );
        })}
      </List>
      <Box sx={{ px: 2, py: 1, borderTop: 1, borderColor: 'divider', display: 'flex', gap: 2 }}>
        {[['↑↓', 'navigate'], ['↵', 'open'], ['1–6', 'sections'], ['/', 'filter'], ['Ctrl+K', 'this palette']].map(([k, v]) => (
          <Typography key={k} variant='caption' color='text.secondary'>
            <Box component='kbd' sx={{ px: 0.6, py: 0.1, border: 1, borderColor: 'divider', borderRadius: 0.5, mr: 0.5, fontFamily: 'inherit' }}>{k}</Box>
            {v}
          </Typography>
        ))}
      </Box>
    </Dialog>
  );
}
