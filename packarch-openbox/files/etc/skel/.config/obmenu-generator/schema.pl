#!/usr/bin/perl

## Copyright (C) 2020-2026 Aditya Shakya <adi1090x@gmail.com>

# obmenu-generator - schema file

require "$ENV{HOME}/.config/obmenu-generator/config.pl";

our $SCHEMA = [
	
    {sep => 'Applications'},
    #       NAME            LABEL          ICON
    {cat => ['utility',     'Accessoires', 'applications-utilities']},
    {cat => ['development', 'Développement', 'applications-development']},
    {cat => ['education',   'Éducation',   'applications-science']},
    {cat => ['game',        'Jeux',       'applications-games']},
    {cat => ['graphics',    'Graphisme',    'applications-graphics']},
    {cat => ['audiovideo',  'Multimedia',  'applications-multimedia']},
    {cat => ['network',     'Internet',     'applications-internet']},
    {cat => ['office',      'Bureautique',      'applications-office']},
    {cat => ['other',       'Autres',       'applications-other']},
    {cat => ['settings',    'Paramètres',    'gnome-settings']},
    {cat => ['system',      'Système',      'applications-system']},

    #{sep},
    #{cat => ['qt',          'QT Applications',    'qt4logo']},
    #{cat => ['gtk',         'GTK Applications',   'gnome-applications']},
    #{cat => ['x_xfce',      'XFCE Applications',  'applications-other']},
    #{cat => ['gnome',       'GNOME Applications', 'gnome-applications']},
    #{cat => ['consoleonly', 'CLI Applications',   'applications-utilities']},
]
