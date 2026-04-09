<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

class EnsureExamUQClientAccess
{
    public function handle(Request $request, Closure $next): Response
    {
        $clientTypeHeader = strtolower((string) $request->header('X-ExamUQ-Client-Type', ''));
        $sourceHeader = strtolower((string) $request->header('X-ExamUQ-Source', ''));
        $userAgent = strtolower((string) $request->userAgent());

        $resolvedClientType = null;

        if (in_array($clientTypeHeader, ['desktop_client', 'chrome_extension'], true)) {
            $resolvedClientType = $clientTypeHeader;
        } elseif ($sourceHeader === 'client') {
            $resolvedClientType = 'desktop_client';
        } elseif ($sourceHeader === 'extension') {
            $resolvedClientType = 'chrome_extension';
        } elseif (str_contains($userAgent, 'examuq-client')) {
            $resolvedClientType = 'desktop_client';
        } elseif (str_contains($userAgent, 'examuq-extension')) {
            $resolvedClientType = 'chrome_extension';
        }

        if (! $resolvedClientType) {
            return response()
                ->view('exam.client-required', [], 403);
        }

        $request->attributes->set('examuqClientType', $resolvedClientType);
        $request->attributes->set('examuqDeviceId', $request->header('X-ExamUQ-Device-Id'));

        return $next($request);
    }
}
