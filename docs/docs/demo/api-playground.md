---
title: API Playground
description: Teste die REST-API direkt im Browser
---

import React from 'react';
import SwaggerUI from 'swagger-ui-react';
import 'swagger-ui-react/swagger-ui.css';
import useBaseUrl from '@docusaurus/useBaseUrl';

# API Playground

<SwaggerUI url={useBaseUrl('/openapi.yaml')} />
