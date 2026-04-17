<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

class EnsureExamUQClientAccess
{
    public function handle(Request $request, Closure $next): Response
    {
        $sessionClientType = null;
        $sessionDeviceId = null;

        if ($request->hasSession()) {
            $sessionClientType = strtolower((string) $request->session()->get('examuq.client_type', ''));
            $sessionDeviceId = $request->session()->get('examuq.device_id');
        }

        $clientTypeHeader = strtolower((string) $request->header('X-ExamUQ-Client-Type', ''));
        $sourceHeader = strtolower((string) $request->header('X-ExamUQ-Source', ''));
        $clientTypeQuery = strtolower((string) $request->query('client_type', ''));
        $sourceQuery = strtolower((string) $request->query('source', ''));
        $userAgent = strtolower((string) $request->userAgent());

        $resolvedClientType = null;

        if (in_array($clientTypeHeader, ['desktop_client', 'chrome_extension'], true)) {
            $resolvedClientType = $clientTypeHeader;
        } elseif (in_array($clientTypeQuery, ['desktop_client', 'chrome_extension'], true)) {
            $resolvedClientType = $clientTypeQuery;
        } elseif ($sourceHeader === 'client') {
            $resolvedClientType = 'desktop_client';
        } elseif ($sourceHeader === 'extension') {
            $resolvedClientType = 'chrome_extension';
        } elseif (in_array($sourceQuery, ['client', 'desktop'], true)) {
            $resolvedClientType = 'desktop_client';
        } elseif ($sourceQuery === 'extension') {
            $resolvedClientType = 'chrome_extension';
        } elseif (in_array($sessionClientType, ['desktop_client', 'chrome_extension'], true)) {
            $resolvedClientType = $sessionClientType;
        } elseif (str_contains($userAgent, 'examuq-client')) {
            $resolvedClientType = 'desktop_client';
        } elseif (str_contains($userAgent, 'examuq-extension')) {
            $resolvedClientType = 'chrome_extension';
        }

        if (! $resolvedClientType) {
            return response()
                ->view('exam.client-required', [], 403);
        }

        $resolvedDeviceId = $request->header('X-ExamUQ-Device-Id')
            ?: $request->query('device_id')
            ?: (is_string($sessionDeviceId) ? $sessionDeviceId : null);

        if ($request->hasSession()) {
            $request->session()->put('examuq.client_type', $resolvedClientType);

            if (is_string($resolvedDeviceId) && $resolvedDeviceId !== '') {
                $request->session()->put('examuq.device_id', $resolvedDeviceId);
            }
        }

        $request->attributes->set('examuqClientType', $resolvedClientType);
        $request->attributes->set('examuqDeviceId', $resolvedDeviceId);

        return $next($request);
    }
}
